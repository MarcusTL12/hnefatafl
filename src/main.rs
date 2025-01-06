#![feature(iter_array_chunks)]

use game::GameState;

mod board;
mod game;

fn main() {
    let mut game = GameState::new();

    game.run();
}
