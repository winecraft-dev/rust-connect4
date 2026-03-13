use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tokio_util::sync::CancellationToken;

use crate::Connection;
use crate::connect4::{Board, BoardState, Color, PlayError};
use crate::game::message::Message;

pub mod message;

#[derive(Debug)]
pub enum GameStatus {
    Playing,
    GameOver,
}

#[derive(Debug, Error)]
pub enum GameError {
    #[error("a connection closed before notification")]
    ConnectionError,
    #[error("the game was cancelled")]
    GameCancelled,
    #[error("state became invalid")]
    WtfState,
}

#[derive(Debug)]
pub struct Game {
    id: usize,
    board: Board,
    cancel: CancellationToken,
    red: Connection,
    blue: Connection,
}

impl Game {
    pub fn new(id: usize, cancel: CancellationToken, red: Connection, blue: Connection) -> Self {
        Self {
            id,
            board: Board::new(),
            cancel,
            red,
            blue,
        }
    }

    // TODO: every expect should return an error that kills the game loop+program

    pub async fn play(&mut self) -> Result<GameStatus, GameError> {
        tokio::select! {
            Some(message) = self.red.recv() => {
                return self.play_message(Color::Red, message);
            }
            Some(message) = self.blue.recv() => {
                return self.play_message(Color::Blue, message);
            }
            _ = self.cancel.cancelled() => {
                // This will cause the game play to stop from the
                // game thread. Afterwhich, the kick function
                // should be called from the game thread
                return Err(GameError::GameCancelled);
            }
        }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    // made this function so I wouldn't have to write code inside that select macro
    // autocompletes are super slow in there
    fn play_message(&mut self, from: Color, msg: Message) -> Result<GameStatus, GameError> {
        let conn = match from {
            Color::Red => &self.red,
            Color::Blue => &self.blue,
        };
        let column = match msg {
            Message::DropChip { column } => column,
            _ => {
                let invalid_message_msg = Message::InvalidMessage;
                if let Err(_) = conn.send(invalid_message_msg) {
                    return Err(GameError::ConnectionError);
                }
                return Ok(GameStatus::Playing);
            }
        };
        match self.board.drop_chip(from, column) {
            Ok(drop_res) => match drop_res.state {
                BoardState::Turn(_) => {
                    if let Err(_) =
                        self.broadcast(Message::moved(&self.board, drop_res.last_move, from))
                    {
                        return Err(GameError::ConnectionError);
                    }
                } // transition to game over state!
                BoardState::Won(winner) => {
                    if let Err(_) = self.broadcast(Message::won(&self.board, winner)) {
                        return Err(GameError::ConnectionError);
                    }
                    return Ok(GameStatus::GameOver);
                }
                BoardState::Stalemate => {
                    if let Err(_) = self.broadcast(Message::stalemate(&self.board)) {
                        return Err(GameError::ConnectionError);
                    }
                    return Ok(GameStatus::GameOver);
                }
            },
            Err(play_err) => {
                let feedback_msg = match play_err {
                    PlayError::GameOver(winner) => Message::won(&self.board, winner),
                    PlayError::Stalemate => Message::stalemate(&self.board),
                    play_err => Message::InvalidMove(play_err),
                };
                if let Err(_) = conn.send(feedback_msg) {
                    return Err(GameError::ConnectionError);
                }
            }
        };
        Ok(GameStatus::Playing)
    }

    fn broadcast(&self, msg: Message) -> Result<(), SendError<Message>> {
        self.red.send(msg.clone())?;
        self.blue.send(msg.clone())?;
        Ok(())
    }

    pub async fn game_start(&self) -> Result<(), SendError<Message>> {
        self.broadcast(Message::board(&self.board))
    }

    pub async fn game_over(&mut self) {
        self.red.close();
        self.blue.close();
    }
}
