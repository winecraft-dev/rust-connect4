use std::cmp;
use std::fmt;
use thiserror::Error;

mod test;

const WIDTH: usize = 7;
const HEIGHT: usize = 6;

pub struct Board {
    chips: [[Option<Color>; HEIGHT]; WIDTH],
    moves: (i32, i32),
    state: BoardState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
    Red,
    Blue,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BoardState {
    Turn(Color),
    Won(Color),
    Stalemate,
}

impl Color {
    fn toggle(&self) -> Self {
        match self {
            Color::Red => Color::Blue,
            Color::Blue => Color::Red,
        }
    }
}

#[derive(Debug, Error)]
pub enum PlayError {
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
            moves: (0, 0),
        }
    }

    pub fn load(layout: &str) -> Result<Board, LoadError> {
        let mut board = Board::new();

        let rows = layout.split('\n');
        if rows.count() != HEIGHT {
            return Err(LoadError::InvalidSize);
        }

        let mut r_moves: i32 = 0;
        let mut b_moves: i32 = 0;
        let mut last_move: Option<(Color, usize, usize)> = None;
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
                        last_move = Some((Color::Red, c, r));
                        Some(Color::Red)
                    }
                    'B' => {
                        b_moves += 1;
                        if last_move.is_some() {
                            return Err(LoadError::ExtraLastMove);
                        }
                        last_move = Some((Color::Blue, c, r));
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
        board.moves = (r_moves, b_moves);

        let last_move = match last_move {
            Some(l) => l,
            None => return Err(LoadError::NoLastMove),
        };

        board.state = BoardState::Turn(last_move.0);
        let win = board.compute_win(last_move.0, (last_move.1, last_move.2));
        board.state = board.compute_state(win);

        Ok(board)
    }

    pub fn drop_chip(&mut self, col: usize) -> Result<BoardState, PlayError> {
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

        let mut chip_loc: Option<(usize, usize)> = None;
        for row in 0..HEIGHT {
            match self.chips[col][row] {
                None => {
                    self.chips[col][row] = Some(current_turn);
                    chip_loc = Some((col, row));
                    break;
                }
                Some(_) => {}
            };
        }

        let Some(chip_loc) = chip_loc else {
            return Err(PlayError::ChipOverflow);
        };

        match current_turn {
            Color::Red => self.moves.0 += 1,
            Color::Blue => self.moves.1 += 1,
        }

        let win = self.compute_win(current_turn, chip_loc);
        self.state = self.compute_state(win);

        Ok(self.state)
    }

    fn compute_state(&self, win: Option<Color>) -> BoardState {
        let board_full = self.moves.0 + self.moves.1 >= (WIDTH * HEIGHT) as i32;
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

    fn compute_win(&self, turn: Color, loc: (usize, usize)) -> Option<Color> {
        let (col, row) = loc;

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
