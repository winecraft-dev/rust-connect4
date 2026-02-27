use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

use crate::Connection;
use crate::connect4::{Board, BoardState, Color, PlayError};
use crate::connection::{ConnRx, ConnectionUpdate};
use crate::game::message::Message;

pub mod message;

#[derive(Debug)]
pub enum GameState {
    AwaitingRed,
    AwaitingBlue,
    Playing,
    GameOver,
}

#[derive(Debug, Error)]
pub enum GameError {
    #[error("new connections stopped")]
    ConnectionUpdateClosed,
    #[error("a connection closed before notification")]
    ConnectionError,
    #[error("state became invalid")]
    WtfState,
}

#[derive(Debug)]
pub struct Game {
    state: GameState,
    conn_rx: ConnRx,
    board: Board,
    red: Option<Connection>,
    blue: Option<Connection>,
}

impl Game {
    pub fn new(conn_rx: ConnRx) -> Self {
        Self {
            state: GameState::AwaitingRed,
            conn_rx: conn_rx,
            board: Board::new(),
            red: None,
            blue: None,
        }
    }

    pub async fn step(&mut self) -> Result<(), GameError> {
        match self.state {
            GameState::AwaitingRed | GameState::AwaitingBlue => self.awaiting_connections().await,
            GameState::Playing => self.play().await,
            GameState::GameOver => self.game_over().await,
        }
    }

    async fn awaiting_connections(&mut self) -> Result<(), GameError> {
        let cu = match self.conn_rx.recv().await {
            None => return Err(GameError::ConnectionUpdateClosed),
            Some(cu) => cu,
        };
        self.state = match cu {
            ConnectionUpdate::Connected(conn) => match self.state {
                GameState::AwaitingRed => {
                    self.red = Some(conn);
                    GameState::AwaitingBlue
                }
                GameState::AwaitingBlue => {
                    self.blue = Some(conn);
                    if let Err(_) = self.broadcast(Message::board(&self.board)) {
                        return Err(GameError::ConnectionError);
                    };
                    GameState::Playing
                }
                _ => return Err(GameError::WtfState),
            },
            ConnectionUpdate::Disconnected(u) => match self.state {
                GameState::AwaitingBlue => {
                    let red = self.red.as_ref().expect("must exist");
                    let state = if red.username.eq(&u) {
                        self.red = None;
                        GameState::AwaitingRed
                    } else {
                        GameState::AwaitingBlue
                    };
                    state
                }
                _ => return Err(GameError::WtfState),
            },
        };
        println!(
            "Game Awaiting... Red: {}, Blue: {}",
            self.red.is_some(),
            self.blue.is_some()
        );
        Ok(())
    }

    // TODO: every expect should return an error that kills the game loop+program

    async fn play(&mut self) -> Result<(), GameError> {
        tokio::select! {
            Some(message) = self.red.as_mut().unwrap().recv() => {
                return self.play_message(Color::Red, message);
            }
            Some(message) = self.blue.as_mut().unwrap().recv() => {
                return self.play_message(Color::Blue, message);
            }
            m = self.conn_rx.recv() => {
                match m {
                    None => return Err(GameError::ConnectionUpdateClosed),
                    Some(cu) => return self.play_connection_update(cu),
                }
            }
        }
    }

    // made this function so I wouldn't have to write code inside that select macro
    // autocompletes are super slow in there
    fn play_message(&mut self, from: Color, msg: Message) -> Result<(), GameError> {
        let conn = match from {
            Color::Red => self.red.as_ref().unwrap(),
            Color::Blue => self.blue.as_ref().unwrap(),
        };
        let column = match msg {
            Message::DropChip { column } => column,
            _ => {
                let invalid_message_msg = Message::InvalidMessage;
                if let Err(_) = conn.send(invalid_message_msg) {
                    return Err(GameError::ConnectionError);
                }
                return Ok(());
            }
        };
        match self.board.drop_chip(from, column) {
            Ok(drop_res) => {
                let broadcast_msg = match drop_res.state {
                    BoardState::Turn(_) => Message::moved(&self.board, drop_res.last_move, from),
                    // transition to game over state!
                    BoardState::Won(winner) => Message::won(&self.board, winner),
                    BoardState::Stalemate => Message::stalemate(&self.board),
                };
                if let Err(_) = self.broadcast(broadcast_msg) {
                    return Err(GameError::ConnectionError);
                }
            }
            Err(play_err) => {
                let feedback_msg = match play_err {
                    PlayError::GameOver(winner) => Message::won(&self.board, winner),
                    PlayError::Stalemate => Message::stalemate(&self.board),
                    play_err => Message::InvalidMove(play_err),
                };
                if let Err(_) = conn.send(feedback_msg) {
                    return Err(GameError::ConnectionError);
                }
            }
        };
        Ok(())
    }

    fn broadcast(&self, msg: Message) -> Result<(), SendError<Message>> {
        self.red.as_ref().unwrap().send(msg.clone())?;
        self.blue.as_ref().unwrap().send(msg.clone())?;
        Ok(())
    }

    fn play_connection_update(&mut self, cu: ConnectionUpdate) -> Result<(), GameError> {
        match cu {
            ConnectionUpdate::Connected(mut conn) => {
                if let Err(_) = conn.send(message::Message::TooManyPlayers) {
                    return Err(GameError::ConnectionError);
                }
                conn.close();
            }
            ConnectionUpdate::Disconnected(username) => {
                let red_disconnected = self.red.as_ref().unwrap().username.eq(&username);
                let blue_disconnected = self.blue.as_ref().unwrap().username.eq(&username);

                if red_disconnected || blue_disconnected {
                    println!("{username} disconnected, cancelling game");
                    self.state = GameState::GameOver;
                }
            }
        }
        Ok(())
    }

    async fn game_over(&mut self) -> Result<(), GameError> {
        println!("Game Over");
        Ok(())
    }
}
