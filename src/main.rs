extern crate core;

mod engine;
mod uci;
mod weights;

use crate::engine::{best_move, evaluation_middlegame, Searcher, best_move_with_time, best_move_infinite_thread};
use chess::Board;
use std::str::FromStr;
use std::time::{Duration, Instant};
use std::thread;
use std::sync::mpsc::{channel, Sender, Receiver};

fn debugging(fen: &str) {
    let board = Board::from_str(fen).expect("Invalid FEN");
    println!(
        "{:?}, {:?}, {:?}",
        evaluation_middlegame(&board),
        best_move(&board, 3).0.to_string(),
        Searcher::new(&board, 3).alpha_beta(-10000, 10000)
    );
}

fn bench() {
    let board = Board::default();
    let mut searcher = Searcher::new(&board, 1);
    searcher.alpha_beta_with_time(Duration::from_secs(2));
    println!("{}", searcher.depth)
}

fn test_infinite() {
    let board = Board::default();
    let (tx, rx) = channel();
    let (tx2, rx2) = channel();
    let mut thread = thread::spawn(move || {
        let mut searcher = Searcher::new(&board, 1);
        searcher.alpha_beta_until_stopped(rx);
        tx2.send( searcher.depth).unwrap();
    });

    thread::sleep(Duration::from_secs(10));
    tx.send(true).unwrap();
    println!("{}", rx2.recv().unwrap());
}

fn find_time_increase_per_depth(){
    let board = Board::default();

    let mut results : Vec<(u64, f64)> = vec![];
    for i in 1..=10 {
        // multiple trials for accuracy
        let mut results_for_depth : Vec<f64> = vec![];
        for a in 1..50 {
            let mut searcher = Searcher::new(&board, 1);
            searcher.depth = i;
            let time = Instant::now();
            searcher.alpha_beta(i32::MIN, i32::MAX);
            let elapsed = time.elapsed();
            results_for_depth.push((elapsed.as_millis() as f64));
            println!("Depth: {}, Time: {:?}", i, elapsed);
        }
        // average
        let mut sum : f64 = 0_f64;
        for time in &results_for_depth {
            sum += *time as f64;
        }
        let avg = sum / results_for_depth.len() as f64;
        results.push((i as u64, avg));
        println!("Depth: {}, Avg Time: {}", i, avg);

    }
    println!("{:?}", results);
}


fn main() {
    uci::main();
}
