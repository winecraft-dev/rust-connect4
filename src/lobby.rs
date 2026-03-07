use std::collections::HashMap;

use thiserror::Error;

use crate::connection::{ConnRx, Connection, ConnectionUpdate};

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
                println!("[Lobby-CU] Player connecting: ${}", &username);
                self.connecting.insert(username, conn);
                self.matchmake();
            }
            ConnectionUpdate::Disconnected(username) => {
                if let Some(_) = self.connecting.remove(&username) {
                    println!("[Lobby-CU] Player disconnected: ${}", username);
                }
            }
        }

        Ok(())
    }

    fn matchmake(&mut self) {
        if self.connecting.len() < 2 {
            return;
        }
        let players = Vec::<Connection>::new();
        let mut player_iter = self.connecting.keys();
        if let Some(username) = player_iter.next() {
            self.connecting.remove_entry(username);
        } else {
        }
    }
}

fn gameplay() {}
