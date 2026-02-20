use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc::{self, error::SendError};

use tokio_util::sync::CancellationToken;
use warp::{ws::Message, ws::WebSocket};

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
    rx: mpsc::UnboundedReceiver<Message>,
    tx: mpsc::UnboundedSender<Message>,
}

impl Connection {
    pub fn send(&mut self, m: Message) -> Result<(), SendError<Message>> {
        self.tx.send(m)
    }

    pub async fn recv(&mut self) -> Option<Message> {
        self.rx.recv().await
    }

    pub fn close(&mut self) {
        self.token.cancel();
    }
}

pub async fn handle_connection(username: String, socket: WebSocket, conn_tx: ConnTx) {
    let (im_tx, im_rx) = mpsc::unbounded_channel::<Message>();
    let (og_tx, mut og_rx) = mpsc::unbounded_channel::<Message>();
    let token = CancellationToken::new();

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

    let og_token = token.child_token();
    tokio::task::spawn(async move {
        loop {
            tokio::select! {
                _ = og_token.cancelled() => {
                    let _ = ws_tx.close().await;
                    break;
                }
                Some(message) = og_rx.recv() => {
                    let _ = ws_tx.send(message).await;
                }
            }
        }
        // println!("Outgoing Messages loop cancelled");
    });

    let im_token = token.child_token();
    loop {
        tokio::select! {
            Some(result) = ws_rx.next() => {
                let msg = match result {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("websocket error(username={}): {}", username, e);
                        break;
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
    // println!("Incoming Messages loop cancelled");

    token.cancel();
    let _ = conn_tx.send(ConnectionUpdate::Disconnected(username));
}
