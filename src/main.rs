extern crate core;

mod engine;
mod uci;
mod weights;

use crate::engine::{best_move, evaluation_middlegame, Searcher};
use chess::Board;
use std::str::FromStr;

fn debugging() {
    let board = Board::from_str("8/8/8/8/8/K7/8/1k1Q4 b - - 0 1").expect("Invalid FEN");
    println!(
        "{:?}, {:?}, {:?}",
        evaluation_middlegame(&board),
        best_move(&board, 3).0.to_string(),
        Searcher::new(&board, 3).alpha_beta(-10000, 10000)
    );
}

fn main() {
    uci::main();
}
