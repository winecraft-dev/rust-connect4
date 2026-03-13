use std::collections::HashMap;

use rand::random_bool;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

use crate::{
    connection::{ConnRx, Connection, ConnectionUpdate},
    game::{Game, GameStatus},
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

    game_counter: usize,
    player_games: HashMap<String, usize>,
    games: HashMap<usize, CancellationToken>,
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

            game_counter: 0,
            player_games: HashMap::new(),
            games: HashMap::new(),
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
                println!("[Lobby] Player \"{}\" connecting", &username);
                self.connecting.insert(username, conn);
                let mmatch = match self.matchmake() {
                    Some(m) => m,
                    None => return Ok(()),
                };
                self.start_match(mmatch);
            }
            ConnectionUpdate::Disconnected(username) => {
                if let Some(_) = self.connecting.remove(&username) {
                    println!("[Lobby] Player \"{}\" disconnected", username);
                }
                if let Some(game_id) = self.player_games.remove(&username) {
                    println!(
                        "[Lobby] Player \"{}\" disconnected from Game \"{}\", cancelling...",
                        &username, game_id,
                    );
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

    fn start_match(&mut self, m: Match) {
        let game_id = self.game_counter;
        self.game_counter += 1;

        let red_username = m.red.username.clone();
        let blue_username = m.blue.username.clone();

        let cancel_token = CancellationToken::new();
        let game = Game::new(game_id, cancel_token.child_token(), m.red, m.blue);

        self.player_games.insert(red_username.clone(), game_id);
        self.player_games.insert(blue_username.clone(), game_id);
        self.games.insert(game_id, cancel_token);

        tokio::task::spawn(async move { gameplay(game).await });

        println!(
            "[Lobby] Starting Game \"{}\"; Red: \"{}\", Blue: \"{}\"",
            game_id, red_username, blue_username
        );
    }
}

async fn gameplay(mut game: Game) {
    let e = game.game_start().await;
    println!("Test {e:?}");
    loop {
        match game.play().await {
            Ok(status) => match status {
                GameStatus::Playing => {
                    println!("Playing")
                }
                GameStatus::GameOver => {
                    break;
                }
            },
            Err(e) => {
                println!("{e}");
                break;
            }
        }
    }
    let _ = game.game_over();
    println!("Game over");
}
