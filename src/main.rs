#![feature(iter_array_chunks)]

use std::env;

use board::{BoardState, HighlightedBoardState, TOWERS};
use game::GameState;

mod board;
mod bot;
mod game;

fn main() {
    let mut args = env::args();

    args.next();

    match args.next().as_deref() {
        Some("test") => {
            // let mut board = BoardState::new();

            // board.set_2d([3, 3], Piece::King);
            // board.set_2d([4, 3], Piece::Black);
            // board.set_2d([3, 4], Piece::Black);

            // board.set_2d([4, 3], Piece::Black);
            // board.set_2d([3, 5], Piece::Black);

            // board.set_2d([4, 4], Piece::White);
            // board.set_2d([0, 5], Piece::White);

            let board = BoardState::standard_setup();

            println!("{}", HighlightedBoardState(board, TOWERS));
        }
        _ => {
            let mut game = GameState::new();

            game.run();
        }
    }
}
