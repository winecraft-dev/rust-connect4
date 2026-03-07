use std::collections::HashMap;

use rand::random_bool;
use thiserror::Error;

use crate::{
    connection::{ConnRx, Connection, ConnectionUpdate},
    game::Game,
};

#[derive(Debug, Error)]
pub enum LobbyError {
    #[error("new connections stopped")]
    ConnectionUpdateClosed,
}

#[derive(Debug)]
pub struct Lobby {
    conn_rx: ConnRx,
    connecting: HashMap<String, Connection>,
}

struct Match {
    red: Connection,
    blue: Connection,
}

impl Lobby {
    pub fn new(conn_rx: ConnRx) -> Self {
        Self {
            conn_rx,
            connecting: HashMap::new(),
        }
    }

    pub async fn lobby(&mut self) -> Result<(), LobbyError> {
        let cu = match self.conn_rx.recv().await {
            None => return Err(LobbyError::ConnectionUpdateClosed),
            Some(cu) => cu,
        };
        match cu {
            ConnectionUpdate::Connected(conn) => {
                let username = conn.username.clone();
                println!("[Lobby] Player connecting: {}", &username);
                self.connecting.insert(username, conn);
                if let Some(mmatch) = self.matchmake() {
                    println!(
                        "[Lobby] Match made! Red: {}, Blue: {}",
                        &mmatch.red.username, &mmatch.blue.username,
                    );
                }
            }
            ConnectionUpdate::Disconnected(username) => {
                if let Some(_) = self.connecting.remove(&username) {
                    println!("[Lobby] Player disconnected: {}", username);
                }
            }
        }

        Ok(())
    }

    fn matchmake(&mut self) -> Option<Match> {
        let mut usernames = self.connecting.keys().cloned();
        let u1 = usernames.next();
        let u2 = usernames.next();
        let (Some(u1), Some(u2)) = (u1, u2) else {
            return None;
        };

        let c1 = self.connecting.remove(&u1);
        let c2 = self.connecting.remove(&u2);
        let (Some(c1), Some(c2)) = (c1, c2) else {
            unreachable!(); // we should panic cause this is impossible
        };

        let (red, blue) = match random_bool(1.0 / 2.0) {
            true => (c1, c2),
            false => (c2, c1),
        };

        return Some(Match { red, blue });
    }
}

fn gameplay() {}
