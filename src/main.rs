use futures_util::{SinkExt, StreamExt, TryFutureExt};
use game::Game;
use tokio::sync::mpsc;
use warp::{Filter, ws, ws::Message};

use crate::connection::Connection;

mod connect4;
mod connection;
mod game;

#[tokio::main]
async fn main() {
    let (ic_s, ic_r) = mpsc::channel::<Connection>(1);
    let mut game = Game::new(ic_r);

    tokio::task::spawn(async move {
        loop {
            game.play().await;
        }
    });

    routes(ic_s).await;
}

async fn routes(ic: mpsc::Sender<Connection>) {
    let ic_filter = warp::any().map(move || ic.clone());

    let ws_play = warp::path!("play" / String)
        .and(warp::ws())
        .and(ic_filter)
        .map(
            |username: String, websocket: ws::Ws, ic: mpsc::Sender<Connection>| {
                websocket.on_upgrade(move |socket| handle_connection(username, socket, ic))
            },
        );

    let index = warp::path::end().map(|| warp::reply::html(INDEX_HTML));
    let routes = index.or(ws_play);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}

const INDEX_HTML: &str = r#"<!DOCTYPE html>
<h1>Hello world</h1>
"#;

async fn handle_connection(
    username: String,
    socket: ws::WebSocket,
    incoming_connections: mpsc::Sender<Connection>,
) {
    let (im_s, im_r) = mpsc::channel::<Message>(1);
    let (om_s, mut om_r) = mpsc::channel::<Message>(1);

    let c = Connection::new(username.as_str(), im_r, om_s);
    incoming_connections.send(c).await.unwrap();

    /*
    let (mut ws_tx, mut ws_rx) = socket.split();

    tokio::task::spawn(async move {
        while let Some(msg) = om_r.recv().await {
            ws_tx
                .send(msg)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });
    */
}
