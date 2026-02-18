use crate::Connection;
use crate::connect4::Board;
use tokio::sync::mpsc;

pub struct Game {
    incoming_connections: mpsc::Receiver<Connection>,
    connect4: Board,
    red: Option<Connection>,
    blue: Option<Connection>,
}

impl Game {
    pub fn new(ic: mpsc::Receiver<Connection>) -> Self {
        Self {
            incoming_connections: ic,
            connect4: Board::new(),
            red: None,
            blue: None,
        }
    }

    pub async fn play(&mut self) {
        if let Some(conn) = self.incoming_connections.recv().await {
            println!("Connection received: {:?}", conn);
        }
    }
}
