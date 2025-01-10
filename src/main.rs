#![feature(iter_array_chunks)]

use std::env;

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
            let board = BoardState::standard_setup();

            let tmp: Vec<_> = board.all_moves(Faction::Black).collect();

            println!("{}", tmp.len());

            println!("{tmp:?}");

            println!(
                "{}",
                tmp.into_iter().map(|x| x.count_ones()).sum::<usize>()
            );
        }
        _ => {
            let mut game = GameState::new();

            game.run();
        }
    }
}
