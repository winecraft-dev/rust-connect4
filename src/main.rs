use game::Game;
use tokio::sync::mpsc;
use warp::{Filter, ws, ws::Message};

use crate::connection::{ConnTx, Connection, ConnectionUpdate};

mod connect4;
mod connection;
mod game;

#[tokio::main]
async fn main() {
    let (ic_tx, ic_rx) = mpsc::unbounded_channel::<ConnectionUpdate>();
    let mut game = Game::new(ic_rx);

    tokio::task::spawn(async move {
        loop {
            game.step().await.expect("Game error");
        }
    });

    routes(ic_tx).await;
}

async fn routes(ic_tx: ConnTx) {
    let ic_filter = warp::any().map(move || ic_tx.clone());

    let ws_play = warp::path!("play" / String)
        .and(warp::ws())
        .and(ic_filter)
        .map(|username: String, w: ws::Ws, ic_tx: ConnTx| {
            w.on_upgrade(move |socket| connection::handle_connection(username, socket, ic_tx))
        });

    let index = warp::path::end().map(|| warp::reply::html(INDEX_HTML));
    let routes = index.or(ws_play);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<h1>Hello world</h1>
"#;
