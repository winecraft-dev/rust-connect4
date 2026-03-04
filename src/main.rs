use game::Game;
use tokio::sync::mpsc;
use warp::{Filter, ws};

use crate::{
    connection::{ConnTx, Connection, ConnectionUpdate},
    game::{GameError, GameStatus, LobbyStatus},
};

mod connect4;
mod connection;
mod game;

#[tokio::main]
async fn main() {
    let (ic_tx, ic_rx) = mpsc::unbounded_channel::<ConnectionUpdate>();
    let mut game = Game::new(ic_rx);

    tokio::task::spawn(async move {
        loop {
            let status = match game.lobby().await {
                Ok(s) => s,
                Err(e) => panic!("{}", e),
            };
            println!("Lobby status: {:?}", status);
            match status {
                LobbyStatus::Ready => break,
                _ => {}
            }
        }
        println!("Starting!");
        let _ = game.game_start().await;
        loop {
            let status = match game.play().await {
                Ok(s) => s,
                Err(e) => match e {
                    GameError::PlayerQuit => break,
                    e => panic!("{}", e),
                },
            };
            println!("Game status: {:?}", status);
            match status {
                GameStatus::GameOver => break,
                _ => {}
            }
        }
        println!("Game over!");
        game.game_over().await;
    });

    routes(ic_tx).await;
}

async fn routes(ic_tx: ConnTx) {
    let ic_filter = warp::any().map(move || ic_tx.clone());

    let static_files = warp::get().and(warp::fs::dir("static"));

    let ws_play = warp::path!("play" / String)
        .and(warp::ws())
        .and(ic_filter)
        .map(|username: String, w: ws::Ws, ic_tx: ConnTx| {
            w.on_upgrade(move |socket| connection::handle_connection(username, socket, ic_tx))
        });

    let routes = static_files.or(ws_play);

    println!("Warp serving");
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}
