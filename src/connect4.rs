use std::fmt;
use thiserror::Error;

const WIDTH: usize = 7;
const HEIGHT: usize = 6;

pub struct Board {
    chips: [[Option<Color>; HEIGHT]; WIDTH],
    state: GameState,
}

#[derive(Clone, Copy, Debug)]
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

        let mut fully_occupied = true;
        for row in 0..HEIGHT {
            match self.chips[col][row] {
                None => {
                    self.chips[col][row] = Some(current_turn);
                    fully_occupied = false;
                    break;
                }
                Some(_) => {}
            };
        }

        if fully_occupied {
            return Err(PlayError::ChipOverflow);
        }

        self.state = self.compute_state();
        Ok(self.state)
    }

    fn compute_state(&self) -> GameState {
        match self.compute_win() {
            None => {
                let GameState::Turn(color) = self.state else {
                    unreachable!();
                };
                GameState::Turn(color.toggle())
            }
            Some(winner) => GameState::Won(winner),
        }
    }

    fn compute_win(&self) -> Option<Color> {
        None
    }

    pub fn state(&self) -> GameState {
        return self.state;
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new();
        output.push_str("Connect4\n");
        output.push_str("+━━━━━━━+\n");
        for row in (0..HEIGHT).rev() {
            output.push('|');
            for col in 0..WIDTH {
                let slot = self.chips[col][row];
                match slot {
                    None => output.push_str("⋅"),
                    Some(chip) => match chip {
                        Color::Red => output.push('⦿'),
                        Color::Blue => output.push('○'),
                    },
                }
            }
            output.push_str("|\n");
        }
        output.push_str("+━━━━━━━+\n");
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
