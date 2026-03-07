use tokio::sync::mpsc;
use warp::{Filter, ws};

use crate::{
    connection::{ConnTx, Connection, ConnectionUpdate},
    lobby::Lobby,
};

mod connect4;
mod connection;
mod game;
mod lobby;

#[tokio::main]
async fn main() {
    let (ic_tx, ic_rx) = mpsc::unbounded_channel::<ConnectionUpdate>();
    let mut lobby = Lobby::new(ic_rx);

    tokio::task::spawn(async move {
        loop {
            match lobby.lobby().await {
                Ok(_) => {}
                Err(e) => panic!("{}", e),
            };
        }
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

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;
}
