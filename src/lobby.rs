use std::collections::HashMap;

use rand::random_bool;
use thiserror::Error;
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

use crate::{
    connection::{ConnRx, Connection, ConnectionUpdate},
    game::{Game, GameStatus, message::Message},
};

#[derive(Debug, Error)]
pub enum LobbyError {
    // fatal
    #[error("channels were closed prematurely")]
    ChannelsClosed,
    #[error("match id is missing")]
    MissingMatchID,
}

#[derive(Debug)]
pub struct Lobby {
    conn_rx: ConnRx,
    connecting: HashMap<String, Connection>,
    playing: HashMap<String, usize>,

    over_tx: MatchOverTx,
    over_rx: MatchOverRx,
    game_counter: usize,
    matches: HashMap<usize, CancellationToken>,
}

struct MatchCandidate {
    red: Connection,
    blue: Connection,
}

#[derive(Debug)]
struct MatchOver {
    id: usize,
    red: String,
    blue: String,
}

type MatchOverTx = mpsc::UnboundedSender<MatchOver>;
type MatchOverRx = mpsc::UnboundedReceiver<MatchOver>;

impl Lobby {
    pub fn new(conn_rx: ConnRx) -> Self {
        let (over_tx, over_rx) = mpsc::unbounded_channel::<MatchOver>();
        Self {
            conn_rx,
            connecting: HashMap::new(),
            playing: HashMap::new(),

            over_tx,
            over_rx,
            game_counter: 0,
            matches: HashMap::new(),
        }
    }

    pub async fn lobby(&mut self) -> Result<(), LobbyError> {
        tokio::select! {
            Some(mo) = self.over_rx.recv() => {
                return self.game_finished(mo).await;
            }
            Some(cu) = self.conn_rx.recv() => {
                return self.player_connection(cu).await;
            }
            else => return Err(LobbyError::ChannelsClosed)
        }
    }

    async fn player_connection(&mut self, cu: ConnectionUpdate) -> Result<(), LobbyError> {
        match cu {
            ConnectionUpdate::Connected(mut conn) => {
                let username = conn.username.clone();
                if self.connecting.contains_key(&username) || self.playing.contains_key(&username) {
                    let _ = conn.send(Message::RepeatUsername);
                    println!(
                        "[Lobby] Player \"{}\" attempted to connect with repeat username",
                        &username
                    );
                    conn.decline();
                    return Ok(());
                }
                conn.accept();
                println!("[Lobby] Player \"{}\" connecting", &username);
                self.connecting.insert(username, conn);
                let mc = match self.matchmake() {
                    Some(mc) => mc,
                    None => return Ok(()),
                };
                self.start_match(mc);
            }
            ConnectionUpdate::Disconnected(username) => {
                if let Some(_) = self.connecting.remove(&username) {
                    println!("[Lobby] Player \"{}\" disconnected", username);
                }
                let game_id = match self.playing.get(&username) {
                    Some(id) => id,
                    None => return Ok(()),
                };
                println!(
                    "[Lobby] Player \"{}\" disconnected from Game \"{}\", cancelling...",
                    &username, game_id,
                );
                let Some(cancel_token) = self.matches.get(game_id) else {
                    return Err(LobbyError::MissingMatchID);
                };
                cancel_token.cancel();
            }
        }

        Ok(())
    }

    async fn game_finished(&mut self, mo: MatchOver) -> Result<(), LobbyError> {
        println!("[Lobby] Game \"{}\" is over", mo.id);
        let _ = self.playing.remove(&mo.red);
        let _ = self.playing.remove(&mo.blue);
        let _ = self.matches.remove(&mo.id);
        Ok(())
    }

    fn matchmake(&mut self) -> Option<MatchCandidate> {
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

        Some(MatchCandidate { red, blue })
    }

    fn start_match(&mut self, mc: MatchCandidate) {
        let id = self.game_counter;
        let cancel_token = CancellationToken::new();
        let red_username = mc.red.username.clone();
        let blue_username = mc.blue.username.clone();

        // Game manages connections
        // MatchOver is just the msg used by the game thread to signal lobby thread
        let game = Game::new(id, cancel_token.child_token(), mc.red, mc.blue);
        let mo = MatchOver {
            id,
            red: red_username.clone(),
            blue: blue_username.clone(),
        };

        self.playing.insert(red_username.clone(), id);
        self.playing.insert(blue_username.clone(), id);
        self.matches.insert(id, cancel_token);
        self.game_counter += 1;

        let over_tx = self.over_tx.clone();
        tokio::task::spawn(async move { gameplay(game, mo, over_tx).await });

        println!(
            "[Lobby] Starting Game \"{}\"; Red: \"{}\", Blue: \"{}\"",
            id, red_username, blue_username
        );
    }
}

// we need a channel to back feed the lobby with Gameplay Results
async fn gameplay(mut game: Game, mo: MatchOver, over_tx: MatchOverTx) {
    match game.game_start().await {
        Err(e) => {
            println!("[Game {}] Failed to start, ending game: {}", game.id(), e);
            game.game_over();
            let _ = over_tx.send(mo);
            return;
        }
        Ok(()) => {}
    }
    loop {
        match game.play().await {
            Ok(status) => match status {
                GameStatus::Playing => {}
                GameStatus::GameWon(winner) => {
                    println!("[Game {}] \"{}\" won", game.id(), winner);
                    break;
                }
                GameStatus::Stalemate => {
                    println!("[Game {}] Ended in stalemate", game.id());
                    break;
                }
            },
            Err(e) => {
                println!("[Game {}] Error: {}", game.id(), e);
                break;
            }
        }
    }
    // sleep(Duration::from_secs(3)).await;
    let _ = over_tx.send(mo);
    game.game_over();
}
