use serde::{Deserialize, Serialize};

use crate::{
    connect4::{BoardState, PlayError, Turn},
    game::Game,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    // input
    DropChip {
        turn: Turn,
        column: usize,
    },

    // output
    InvalidFormat,
    Board {
        state: BoardState,
        moves: Turn,
        board: String,
    },
    InvalidMove(PlayError),
    WaitYourTurn,

    TooManyPlayers,
}
