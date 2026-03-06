use serde::{Deserialize, Serialize};

use crate::connect4::{BoardLayout, Color, Move, PlayError};

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
        board: BoardLayout,
    },
    Moved {
        last_mover: Color,
        last_move: Move,
        board: BoardLayout,
    },
    Won {
        winner: Color,
        last_move: Move,
        board: BoardLayout,
    },
    Stalemate {
        last_move: Move,
        board: BoardLayout,
    },
    InvalidFormat,
    InvalidMessage,
    InvalidMove(PlayError),

    TooManyPlayers,
}
