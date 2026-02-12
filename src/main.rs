use crate::connect4::{Board, GameState};
use std::{error::Error, io};

mod connect4;

fn main() -> Result<(), Box<dyn Error>> {
    let mut game = Board::new();

    loop {
        println!("{}", game);

        let mut player_move = String::new();
        io::stdin()
            .read_line(&mut player_move)
            .expect("failed to read stdin");
        let player_move: usize = match player_move.trim().parse() {
            Ok(c) => c,
            Err(_) => {
                println!("Please enter a number");
                continue;
            }
        };

        let result = game.drop_chip(player_move);
        match result {
            Ok(state) => match state {
                GameState::Won(winner) => {
                    println!("{:?} won the game!", winner);
                    println!("{}", game);
                    return Ok(());
                }
                GameState::Stalemate => {
                    println!("Game ended in Stalemate!");
                    println!("{}", game);
                    return Ok(());
                }
                _ => {}
            },
            Err(e) => {
                println!("Problem with gameplay, {e:?}");
            }
        }
    }
}
