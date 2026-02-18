use crate::connect4::{Board, BoardState, Color};

#[test]
fn test_win_vertical() {
    let layout = r#".......
.......
R......
rb.....
rb.....
rb....."#;

    let board = Board::load(layout);
    let board = board.unwrap();
    println!("{board}");
    assert_eq!(board.state, BoardState::Won(Color::Red));
}

#[test]
fn test_win_horizontal() {
    let layout = r#".......
.......
.......
.......
...bb.b
...rrRr"#;

    let board = Board::load(layout);
    let board = board.unwrap();
    println!("{board}");
    assert_eq!(board.state, BoardState::Won(Color::Red));
}

#[test]
fn test_win_diagonal_inverse() {
    let layout = r#".......
r......
bR.....
brr....
rbbr...
rbbb..."#;

    let board = Board::load(layout);
    let board = board.unwrap();
    println!("{board}");
    assert_eq!(board.state, BoardState::Won(Color::Red));
}

#[test]
fn test_win_diagonal() {
    let layout = r#".......
.......
......r
.....rb
....rbb
...Rrbb"#;

    let board = Board::load(layout);
    let board = board.unwrap();
    println!("{board}");
    assert_eq!(board.state, BoardState::Won(Color::Red));
}
