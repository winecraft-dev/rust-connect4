use std::cmp;
use std::fmt;
use thiserror::Error;

const WIDTH: usize = 7;
const HEIGHT: usize = 6;

pub struct Board {
    chips: [[Option<Color>; HEIGHT]; WIDTH],
    state: GameState,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Color {
    Red,
    Blue,
}

#[derive(Clone, Copy, Debug)]
pub enum GameState {
    Turn(Color),
    Won(Color),
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
}

impl Board {
    pub fn new() -> Board {
        Board {
            chips: [[Option::None; HEIGHT]; WIDTH],
            state: GameState::Turn(Color::Red),
        }
    }

    pub fn drop_chip(&mut self, col: usize) -> Result<GameState, PlayError> {
        match col {
            0..WIDTH => {}
            _ => return Err(PlayError::OutOfRange),
        };

        let current_turn = match self.state {
            GameState::Turn(c) => c,
            GameState::Won(winner) => {
                return Err(PlayError::GameOver(winner));
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

        let win = self.compute_win(current_turn, chip_loc);
        self.state = self.compute_state(win);

        Ok(self.state)
    }

    fn compute_state(&self, win: Option<Color>) -> GameState {
        match win {
            None => {
                let GameState::Turn(color) = self.state else {
                    unreachable!();
                };
                GameState::Turn(color.toggle())
            }
            Some(winner) => GameState::Won(winner),
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
                    None => output.push_str("⋅ "),
                    Some(chip) => match chip {
                        Color::Red => output.push_str("⦿ "),
                        Color::Blue => output.push_str("○ "),
                    },
                }
            }
            output.push_str("|\n");
        }
        output.push_str("+━━━━━━━━━━━━━━━+\n");
        match self.state {
            GameState::Turn(current) => {
                let turn_message = format!("Turn: {:?}", current);
                output.push_str(turn_message.as_str());
            }
            GameState::Won(winner) => {
                let win_message = format!("Winner: {:?}", winner);
                output.push_str(win_message.as_str());
            }
        }
        write!(f, "{}", output)
    }
}
