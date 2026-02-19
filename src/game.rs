use crate::Connection;
use crate::connect4::Board;
use crate::connection::ConnRx;

pub struct Game {
    conn_rx: ConnRx,
    connect4: Board,
    red: Option<Connection>,
    blue: Option<Connection>,
}

impl Game {
    pub fn new(conn_rx: ConnRx) -> Self {
        Self {
            conn_rx: conn_rx,
            connect4: Board::new(),
            red: None,
            blue: None,
        }
    }

    pub async fn play(&mut self) {
        if let Some(conn) = self.conn_rx.recv().await {
            println!("Connection update! {:?}", conn);
        }
    }
}
