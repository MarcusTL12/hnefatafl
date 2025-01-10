#![feature(iter_array_chunks)]

use std::{env, time::Instant};

use board::{BoardState, Faction};
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

            println!("{board}");

            let n: u32 = args.next().map(|x| x.parse().unwrap()).unwrap_or(0);

            let t = Instant::now();
            let m = board.best_move(Faction::Black, n);
            let t = t.elapsed();

            println!("Best move: {m:?}, took: {t:.2?}");
        }
        _ => {
            let mut game = GameState::new();

            game.run();
        }
    }
}
