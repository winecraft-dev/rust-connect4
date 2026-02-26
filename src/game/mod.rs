use thiserror::Error;

use crate::Connection;
use crate::connect4::{Board, Color};
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
    #[error("red player disconnected during play")]
    RedDisconnected,
    #[error("blue player disconnected during play")]
    BlueDisconnected,
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
        println!("Game Awaiting...");
        println!("Red: {:?}", self.red);
        println!("Blue: {:?}", self.blue);
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
                    Some(cu) => self.play_connection_update(cu),
                }
            }
        }

        println!("Game playing!");
        Ok(())
    }

    // made this function so I wouldn't have to write code inside that select macro
    // autocompletes are super slow in there
    fn play_message(
        &mut self,
        from: Color,
        // conn: &Connection,
        m: Message,
    ) -> Result<(), GameError> {
        let conn = match from {
            Color::Red => self.red.as_ref().unwrap(),
            Color::Blue => self.blue.as_ref().unwrap(),
        };
        Ok(())
    }

    fn play_connection_update(&mut self, cu: ConnectionUpdate) {
        match cu {
            ConnectionUpdate::Connected(mut conn) => {
                let _ = conn.send(message::Message::TooManyPlayers);
                conn.close();
            }
            ConnectionUpdate::Disconnected(username) => {
                // handle player disconecting during game
                println!("{username} disconnected");
            }
        }
    }

    async fn game_over(&mut self) -> Result<(), GameError> {
        Ok(())
    }
}
