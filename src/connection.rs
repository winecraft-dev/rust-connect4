use tokio::sync::mpsc;

use warp::ws::Message;

#[derive(Debug)]
pub struct Connection {
    username: String,
    //recv: mpsc::Receiver<Message>,
    //send: mpsc::Sender<Message>,
}

impl Connection {
    pub fn new(u: &str, r: mpsc::Receiver<Message>, s: mpsc::Sender<Message>) -> Self {
        Connection {
            username: u.to_string(),
            //recv: r,
            //send: s,
        }
    }
}
