use serde::{Deserialize, Serialize};

use crate::{
    connect4::{BoardState, Color, Move, PlayError, Turn},
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
        turn: Color,
        moves: Turn,
        board: String,
    },
    Moved {
        turn: Color,
        last_move: Move,
        moves: Turn,
        board: String,
    },
    Won {
        winner: Color,
        last_move: Move,
        moves: Turn,
        board: String,
    },
    Stalemate {
        last_move: Move,
        moves: Turn,
        board: String,
    },
    InvalidMove(PlayError),
    WaitYourTurn,

    TooManyPlayers,
}
