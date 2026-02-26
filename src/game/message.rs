use serde::{Deserialize, Serialize};

use crate::{
    connect4::{BoardState, Color, Move, PlayError, Turn},
    game::Game,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Message {
    // input
    DropChip {
        column: usize,
    },

    // output
    Board {
        turn: Color,
        board: String,
    },
    Moved {
        last_mover: Color,
        last_move: Move,
        board: String,
    },
    Won {
        winner: Color,
        last_move: Move,
        board: String,
    },
    Stalemate {
        last_move: Move,
        board: String,
    },
    InvalidFormat,
    InvalidMessage,
    InvalidMove(PlayError),

    TooManyPlayers,
}
