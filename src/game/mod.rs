use thiserror::Error;
use tokio::sync::mpsc::error::SendError;

use crate::Connection;
use crate::connect4::{Board, BoardState, Color, PlayError};
use crate::connection::{ConnRx, ConnectionUpdate};
use crate::game::message::Message;

pub mod message;

// TODO: MASSIVE REFACTOR TO RETURN OUT GAME STATE!
//
// currently, the state has two "modes"... it would be more visible if we split up the "step" function
// into two discrete functions. The first "lobby" is for collecting enough users to start the game. The second
// "play" is for stepping the game. In fact, it would be better if those were separate modules with connections
// funneled from the "lobby" to the "play". But for now, let's take out those two modes and have them be their
// own function calls. The main loop will move from "lobby" to "play" mode.

#[derive(Debug)]
pub enum GameStatus {
    Playing,
    GameOver,
}

#[derive(Debug)]
pub enum LobbyStatus {
    AwaitingRed,
    AwaitingBlue,
    Ready,
}

#[derive(Debug, Error)]
pub enum GameError {
    #[error("new connections stopped")]
    ConnectionUpdateClosed,
    #[error("a connection closed before notification")]
    ConnectionError,
    #[error("a player quit the game")]
    PlayerQuit,
    #[error("state became invalid")]
    WtfState,
}

#[derive(Debug)]
pub struct Game {
    conn_rx: ConnRx,
    board: Board,
    red: Option<Connection>,
    blue: Option<Connection>,
}

impl Game {
    pub fn new(conn_rx: ConnRx) -> Self {
        Self {
            conn_rx: conn_rx,
            board: Board::new(),
            red: None,
            blue: None,
        }
    }

    pub async fn lobby(&mut self) -> Result<LobbyStatus, GameError> {
        let cu = match self.conn_rx.recv().await {
            None => return Err(GameError::ConnectionUpdateClosed),
            Some(cu) => cu,
        };
        match cu {
            ConnectionUpdate::Connected(conn) => {
                if self.red.is_none() {
                    self.red = Some(conn);
                    Ok(LobbyStatus::AwaitingBlue)
                } else {
                    self.blue = Some(conn);
                    Ok(LobbyStatus::Ready)
                }
            }
            ConnectionUpdate::Disconnected(username) => {
                if let Some(red) = self.red.as_ref() {
                    if red.username.eq(&username) {
                        self.red = None;
                        Ok(LobbyStatus::AwaitingRed)
                    } else {
                        Ok(LobbyStatus::AwaitingBlue)
                    }
                } else {
                    Err(GameError::WtfState)
                }
            }
        }
    }

    // TODO: every expect should return an error that kills the game loop+program

    pub async fn play(&mut self) -> Result<GameStatus, GameError> {
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
    fn play_message(&mut self, from: Color, msg: Message) -> Result<GameStatus, GameError> {
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
                return Ok(GameStatus::Playing);
            }
        };
        match self.board.drop_chip(from, column) {
            Ok(drop_res) => match drop_res.state {
                BoardState::Turn(_) => {
                    if let Err(_) =
                        self.broadcast(Message::moved(&self.board, drop_res.last_move, from))
                    {
                        return Err(GameError::ConnectionError);
                    }
                } // transition to game over state!
                BoardState::Won(winner) => {
                    if let Err(_) = self.broadcast(Message::won(&self.board, winner)) {
                        return Err(GameError::ConnectionError);
                    }
                    return Ok(GameStatus::GameOver);
                }
                BoardState::Stalemate => {
                    if let Err(_) = self.broadcast(Message::stalemate(&self.board)) {
                        return Err(GameError::ConnectionError);
                    }
                    return Ok(GameStatus::GameOver);
                }
            },
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
        Ok(GameStatus::Playing)
    }

    fn broadcast(&self, msg: Message) -> Result<(), SendError<Message>> {
        self.red.as_ref().unwrap().send(msg.clone())?;
        self.blue.as_ref().unwrap().send(msg.clone())?;
        Ok(())
    }

    fn play_connection_update(&mut self, cu: ConnectionUpdate) -> Result<GameStatus, GameError> {
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
                    return Err(GameError::PlayerQuit);
                }
            }
        }
        Ok(GameStatus::Playing)
    }

    pub async fn game_start(&self) -> Result<(), SendError<Message>> {
        self.broadcast(Message::board(&self.board))
    }

    pub async fn game_over(&mut self) {
        if let Some(red) = self.red.as_mut() {
            red.close();
        }
        if let Some(blue) = self.blue.as_mut() {
            blue.close();
        }
    }
}
