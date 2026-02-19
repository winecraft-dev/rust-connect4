use futures_util::StreamExt;
use tokio::sync::mpsc;

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
    username: String,
    recv: mpsc::UnboundedReceiver<Message>,
    send: mpsc::UnboundedSender<Message>,
}

impl Connection {
    pub fn new(
        u: &str,
        r: mpsc::UnboundedReceiver<Message>,
        s: mpsc::UnboundedSender<Message>,
    ) -> Self {
        Connection {
            username: u.to_string(),
            recv: r,
            send: s,
        }
    }
}

pub async fn handle_connection(username: String, socket: WebSocket, conn_tx: ConnTx) {
    let (im_tx, im_rx) = mpsc::unbounded_channel::<Message>();
    let (og_tx, og_rx) = mpsc::unbounded_channel::<Message>();

    let conn = Connection::new(username.as_str(), im_rx, og_tx);
    conn_tx.send(ConnectionUpdate::Connected((conn)));

    let (mut ws_tx, mut ws_rx) = socket.split();

    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(username={}): {}", username, e);
                break;
            }
        };
        println!("{} sent message: {:?}", username, msg);
    }

    conn_tx.send(ConnectionUpdate::Disconnected(username));
}
