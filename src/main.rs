#![feature(iter_array_chunks)]

use board::{BoardState, Faction};

mod board;
mod bot;
mod game;

fn main() {
    // let mut game = GameState::new();

    // game.run();

    let board = BoardState::standard_setup();

    let tmp: Vec<_> = board.all_moves(Faction::Black).collect();

    println!("{}", tmp.len());

    println!("{tmp:?}");

    println!("{}", tmp.into_iter().map(|x| x.count_ones()).sum::<usize>());
}
