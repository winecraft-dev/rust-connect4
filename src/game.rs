use std::time::Duration;

use tokio::time::sleep;
use warp::filters::ws;

use crate::Connection;
use crate::connect4::Board;
use crate::connection::{ConnRx, ConnectionUpdate};

#[derive(Debug)]
pub enum GameState {
    AwaitingRed,
    AwaitingBlue,
    Playing,
}

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

    pub async fn step(&mut self) {
        match self.state {
            GameState::AwaitingRed | GameState::AwaitingBlue => self.awaiting_connections().await,
            GameState::Playing => self.play().await,
        };
    }

    async fn awaiting_connections(&mut self) {
        let cu = self.conn_rx.recv().await.expect("everything is dying");
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
                _ => unreachable!(),
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
                _ => unreachable!(),
            },
        };
        println!("Game Awaiting...");
        println!("Red: {:?}", self.red);
        println!("Blue: {:?}", self.blue);
    }

    async fn play(&mut self) {
        sleep(Duration::from_secs(1)).await;
        println!("Game playing!");
    }

    fn player_connect(c: Connection) {}
}
