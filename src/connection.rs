use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::{self, error::SendError};

use crate::game::message::Message as GameMessage;
use tokio_util::sync::CancellationToken;
use warp::{ws::Message as WsMessage, ws::WebSocket};

pub type ConnTx = mpsc::UnboundedSender<ConnectionUpdate>;
pub type ConnRx = mpsc::UnboundedReceiver<ConnectionUpdate>;

#[derive(Debug)]
pub enum ConnectionUpdate {
    Connected(Connection),
    Disconnected(String),
}

#[derive(Debug)]
pub struct Connection {
    pub username: String,
    token: CancellationToken,
    rx: mpsc::UnboundedReceiver<GameMessage>,
    tx: mpsc::UnboundedSender<GameMessage>,
}

impl Connection {
    pub fn send(&self, m: GameMessage) -> Result<(), SendError<GameMessage>> {
        self.tx.send(m)
    }

    pub async fn recv(&mut self) -> Option<GameMessage> {
        self.rx.recv().await
    }

    pub fn close(&mut self) {
        self.token.cancel();
    }
}

pub async fn handle_connection(username: String, socket: WebSocket, conn_tx: ConnTx) {
    let (im_tx, im_rx) = mpsc::unbounded_channel::<GameMessage>();
    let (og_tx, mut og_rx) = mpsc::unbounded_channel::<GameMessage>();
    let token = CancellationToken::new();

    let og_tx_2 = og_tx.clone();

    let conn = Connection {
        username: username.clone(),
        token: token.clone(),
        rx: im_rx,
        tx: og_tx,
    };
    match conn_tx.send(ConnectionUpdate::Connected(conn)) {
        Err(_) => {
            let _ = socket.close().await;
            return;
        }
        Ok(()) => {}
    };

    let (mut ws_tx, mut ws_rx) = socket.split();

    let og_token = token.clone();
    tokio::task::spawn(async move {
        loop {
            tokio::select! {
                _ = og_token.cancelled() => {
                    let _ = ws_tx.close().await;
                    break;
                }
                Some(message) = og_rx.recv() => {
                    let text = match serde_json::to_string(&message) {
                        Ok(t) => t,
                        Err(e) => {
                            eprintln!("serde error: {}", e);
                            continue;
                        }
                    };
                    if let Err(e) = ws_tx.send(WsMessage::text(text)).await {
                        eprintln!("websocket send error: {}", e);
                        og_token.cancel();
                        break;
                    };
                }
            }
        }
        println!("Outgoing Messages loop cancelled");
    });

    let im_token = token.child_token();
    loop {
        tokio::select! {
            Some(result) = ws_rx.next() => {
                let raw_msg = match result {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("websocket receive error: {}", e);
                        break;
                    }
                };
                let text = match raw_msg.to_str() {
                    Ok(t) => t,
                    Err(_) => {
                        let _ = og_tx_2.send(GameMessage::InvalidFormat);
                        continue;
                    }
                };
                let msg = match serde_json::from_str::<GameMessage>(text) {
                    Ok(m) => m,
                    Err(_) => {
                        println!("{}", text);
                        let _ = og_tx_2.send(GameMessage::InvalidFormat);
                        continue;
                    }
                };


                if im_tx.send(msg).is_err() {
                    break;
                };
            }
            _ = im_token.cancelled() => {
                break;
            }
        }
    }
    println!("Incoming Messages loop cancelled");

    token.cancel();
    let _ = conn_tx.send(ConnectionUpdate::Disconnected(username));
}
