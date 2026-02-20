use std::time::Duration;

use thiserror::Error;
use tokio::time::sleep;
use warp::filters::ws::{self, Message};

use crate::Connection;
use crate::connect4::Board;
use crate::connection::{ConnRx, ConnectionUpdate};

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
    connect4: Board,
    red: Option<Connection>,
    blue: Option<Connection>,
}

impl Game {
    pub fn new(conn_rx: ConnRx) -> Self {
        Self {
            state: GameState::AwaitingRed,
            conn_rx: conn_rx,
            connect4: Board::new(),
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
        let red = self.red.as_mut().expect("impossible");
        let blue = self.blue.as_mut().expect("impossible");

        tokio::select! {
            Some(message) = red.recv() => {
                println!("Red msg: {:?}", message);
            }
            Some(message) = blue.recv() => {
                println!("Blue msg: {:?}", message);
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

    fn play_connection_update(&mut self, cu: ConnectionUpdate) {
        match cu {
            ConnectionUpdate::Connected(mut conn) => {
                let _ = conn.send(Message::text("Too many players!"));
                conn.close();
            }
            ConnectionUpdate::Disconnected(username) => {
                println!("{username} disconnected");
            }
        }
    }

    async fn game_over(&mut self) -> Result<(), GameError> {
        Ok(())
    }
}
