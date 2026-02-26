use serde::Deserialize;
use serde::Serialize;
use std::cmp;
use std::fmt;
use thiserror::Error;

use crate::game::message::Message;

mod test;

const WIDTH: usize = 7;
const HEIGHT: usize = 6;

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy)]
pub struct Turn {
    red: i32,
    blue: i32,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Move {
    color: Color,
    row: usize,
    col: usize,
}

#[derive(Debug)]
pub struct Board {
    chips: [[Option<Color>; HEIGHT]; WIDTH],
    moves: Turn,
    last_move: Option<Move>,
    state: BoardState,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Color {
    #[default]
    Red,
    Blue,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum BoardState {
    Turn(Color),
    Won(Color),
    Stalemate,
}

#[derive(Debug)]
pub struct DropResult {
    pub last_move: Move,
    pub state: BoardState,
}

impl Color {
    fn toggle(&self) -> Self {
        match self {
            Color::Red => Color::Blue,
            Color::Blue => Color::Red,
        }
    }
}

impl Message {
    pub fn board(b: &Board) -> Self {
        let BoardState::Turn(turn) = b.state else {
            unreachable!();
        };
        Message::Board {
            turn: turn,
            board: format!("{}", b),
        }
    }

    pub fn won(b: &Board, winner: Color) -> Self {
        Message::Won {
            winner,
            last_move: b.last_move.unwrap(),
            board: format!("{}", b),
        }
    }

    pub fn stalemate(b: &Board) -> Self {
        Message::Stalemate {
            last_move: b.last_move.unwrap(),
            board: format!("{}", b),
        }
    }

    pub fn moved(b: &Board, last_move: Move, mover: Color) -> Self {
        Message::Moved {
            last_mover: mover,
            last_move: last_move,
            board: format!("{}", b),
        }
    }
}

#[derive(Clone, Debug, Error, Deserialize, Serialize)]
pub enum PlayError {
    #[error("wrong color chip")]
    WrongColorChip,
    #[error("move outside of board")]
    OutOfRange,
    #[error("too many chips in column")]
    ChipOverflow,
    #[error("game already finished, winner {0:?}")]
    GameOver(Color),
    #[error("game already ended in stalemate")]
    Stalemate,
}

#[derive(Debug)]
pub enum LoadError {
    InvalidSize,
    InvalidText,
    InvalidMoves,
    NoLastMove,
    ExtraLastMove,
}

impl Board {
    pub fn new() -> Board {
        Board {
            chips: [[Option::None; HEIGHT]; WIDTH],
            state: BoardState::Turn(Color::Red),
            last_move: None,
            moves: Turn::default(),
        }
    }

    #[allow(unused)] // used by tests
    pub fn load(layout: &str) -> Result<Board, LoadError> {
        let mut board = Board::new();

        let rows = layout.split('\n');
        if rows.count() != HEIGHT {
            return Err(LoadError::InvalidSize);
        }

        let mut r_moves: i32 = 0;
        let mut b_moves: i32 = 0;
        let mut last_move: Option<Move> = None;
        for (r_inv, row) in layout.split('\n').enumerate() {
            let r = (HEIGHT - 1) - r_inv;
            if row.len() != WIDTH {
                return Err(LoadError::InvalidSize);
            }
            for (c, color) in row.chars().into_iter().enumerate() {
                board.chips[c][r] = match color {
                    'r' => {
                        r_moves += 1;
                        Some(Color::Red)
                    }
                    'b' => {
                        b_moves += 1;
                        Some(Color::Blue)
                    }
                    'R' => {
                        r_moves += 1;
                        if last_move.is_some() {
                            return Err(LoadError::ExtraLastMove);
                        }
                        last_move = Some(Move {
                            color: Color::Red,
                            row: r,
                            col: c,
                        });
                        Some(Color::Red)
                    }
                    'B' => {
                        b_moves += 1;
                        if last_move.is_some() {
                            return Err(LoadError::ExtraLastMove);
                        }
                        last_move = Some(Move {
                            color: Color::Blue,
                            row: r,
                            col: c,
                        });
                        Some(Color::Blue)
                    }
                    '.' => None,
                    _ => return Err(LoadError::InvalidText),
                };
            }
        }

        let move_difference = r_moves.abs_diff(b_moves);
        if move_difference > 1 {
            return Err(LoadError::InvalidMoves);
        }
        board.moves = Turn {
            red: r_moves,
            blue: b_moves,
        };

        let last_move = match last_move {
            Some(l) => l,
            None => return Err(LoadError::NoLastMove),
        };

        board.last_move = Some(last_move);
        board.state = BoardState::Turn(last_move.color);
        let win = board.compute_win(last_move);
        board.state = board.compute_state(win);

        Ok(board)
    }

    pub fn drop_chip(&mut self, chip: Color, col: usize) -> Result<DropResult, PlayError> {
        match col {
            0..WIDTH => {}
            _ => return Err(PlayError::OutOfRange),
        };

        let current_turn = match self.state {
            BoardState::Turn(c) => c,
            BoardState::Won(winner) => {
                return Err(PlayError::GameOver(winner));
            }
            BoardState::Stalemate => {
                return Err(PlayError::Stalemate);
            }
        };

        if chip.ne(&current_turn) {
            return Err(PlayError::WrongColorChip);
        }

        let mut current_move: Option<Move> = None;
        for row in 0..HEIGHT {
            match self.chips[col][row] {
                None => {
                    self.chips[col][row] = Some(current_turn);
                    current_move = Some(Move {
                        color: current_turn,
                        col: col,
                        row: row,
                    });
                    break;
                }
                Some(_) => {}
            };
        }

        let Some(current_move) = current_move else {
            return Err(PlayError::ChipOverflow);
        };

        match current_turn {
            Color::Red => self.moves.red += 1,
            Color::Blue => self.moves.blue += 1,
        }

        let win = self.compute_win(current_move);
        self.state = self.compute_state(win);
        self.last_move = Some(current_move);

        Ok(DropResult {
            last_move: current_move,
            state: self.state,
        })
    }

    fn compute_state(&self, win: Option<Color>) -> BoardState {
        let board_full = self.moves.red + self.moves.blue >= (WIDTH * HEIGHT) as i32;
        match win {
            None => {
                if board_full {
                    return BoardState::Stalemate;
                }
                let BoardState::Turn(color) = self.state else {
                    unreachable!();
                };
                BoardState::Turn(color.toggle())
            }
            Some(winner) => BoardState::Won(winner),
        }
    }

    fn compute_win(&self, last_move: Move) -> Option<Color> {
        let turn = last_move.color;
        let col = last_move.col;
        let row = last_move.row;

        // horizontal
        let mut current_length = 0;
        let row_lo = row - cmp::min(4, row);
        let row_hi = row + cmp::min(4, HEIGHT - row);
        for r in row_lo..row_hi {
            if count_length(&mut current_length, turn, self.chips[col][r]) {
                return Some(turn);
            }
        }

        // vertical
        let mut current_length = 0;
        let col_lo = col - cmp::min(4, col);
        let col_hi = col + cmp::min(4, WIDTH - col);
        for c in col_lo..col_hi {
            if count_length(&mut current_length, turn, self.chips[c][row]) {
                return Some(turn);
            }
        }

        // diagonal
        let mut current_length = 0;
        let dist = cmp::min(3, cmp::min(col, row));
        let inv_dist = cmp::min(3, cmp::min((HEIGHT - 1) - row, (WIDTH - 1) - col));
        let d_row = row - dist;
        let d_col = col - dist;
        // println!("Calculating with: [{}][{}]", col, row);
        // println!("Row: {row_lo} {row_hi}");
        // println!("Col: {col_lo} {col_hi}");
        // println!("Diag Lo: {dist}+{inv_dist} [{d_col}][{d_row}]");

        for d in 0..(dist + inv_dist + 1) {
            let r = d_row + d;
            let c = d_col + d;
            // println!("[{c}][{r}]");
            if count_length(&mut current_length, turn, self.chips[c][r]) {
                return Some(turn);
            }
        }

        // diagonal negative
        let mut current_length = 0;
        let dist = cmp::min(3, cmp::min(col, (HEIGHT - 1) - row));
        let inv_dist = cmp::min(3, cmp::min((WIDTH - 1) - col, row));
        let d_row = row + dist;
        let d_col = col - dist;
        // println!("Diag I Lo: {dist}+{inv_dist} [{d_col}][{d_row}]");
        for d in 0..(dist + inv_dist + 1) {
            let r = d_row - d;
            let c = d_col + d;
            // println!("[{c}][{r}]");
            if count_length(&mut current_length, turn, self.chips[c][r]) {
                return Some(turn);
            }
        }

        None
    }
}

fn count_length(current_length: &mut i32, turn: Color, chip: Option<Color>) -> bool {
    if let Some(color) = chip {
        if color == turn {
            *current_length += 1;
            return *current_length >= 4;
        }
    }
    *current_length = 0;
    false
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        output.push_str("Connect4\n");
        output.push_str("+━━━━━━━━━━━━━━━+\n");
        for row in (0..HEIGHT).rev() {
            output.push_str("| ");
            for col in 0..WIDTH {
                let slot = self.chips[col][row];
                match slot {
                    None => output.push_str("- "),
                    Some(chip) => match chip {
                        Color::Red => output.push_str("r "),
                        Color::Blue => output.push_str("b "),
                    },
                }
            }
            output.push_str("|\n");
        }
        output.push_str("+━━━━━━━━━━━━━━━+\n");
        match self.state {
            BoardState::Turn(current) => {
                let turn_message = format!("Turn: {:?}", current);
                output.push_str(turn_message.as_str());
            }
            BoardState::Won(winner) => {
                let win_message = format!("Winner: {:?}", winner);
                output.push_str(win_message.as_str());
            }
            BoardState::Stalemate => {
                output.push_str("Stalemate :/");
            }
        }
        write!(f, "{}", output)
    }
}
