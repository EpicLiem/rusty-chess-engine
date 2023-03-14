use crate::engine::{best_move, best_move_with_time, evaluation_middlegame, Searcher};
use chess::{Board, ChessMove};
use std::str::FromStr;

// allow for uci communication

struct Uci {
    board: Board,
    time: u64,
    inc: u64,
    movestogo: u64,
    depth: u64,
    nodes: u64,
    mate: u64,
    movetime: u64,
    infinite: bool,
    searching: bool,
}

struct Listener {
    uci: Uci,
}

impl Listener {
    fn new() -> Listener {
        Listener {
            uci: Uci {
                board: Board::default(),
                time: 0,
                inc: 0,
                movestogo: 0,
                depth: 0,
                nodes: 0,
                mate: 0,
                movetime: 0,
                infinite: false,
                searching: false,
            },
        }
    }

    fn handle(&mut self, line: &str) {
        let mut args = line.split_whitespace();
        let command = args.next().unwrap_or("");
        match command {
            "uci" => self.uci(),
            "isready" => self.isready(),
            "ucinewgame" => self.ucinewgame(),
            "position" => self.position(args),
            "go" => self.go(args),
            "stop" => self.stop(),
            "quit" => self.quit(),
            _ => (),
        }
    }

    fn uci(&mut self) {
        println!("id name Rusty");
        println!("id author Rusty");
        println!("uciok");
    }

    fn isready(&mut self) {
        println!("readyok");
    }

    fn ucinewgame(&mut self) {
        self.uci.board = Board::default();
    }

    fn position(&mut self, mut args: std::str::SplitWhitespace) {
        let mut fen = String::new();
        let mut moves = Vec::new();
        let mut next = args.next().unwrap_or("");
        while next != "" {
            if next == "startpos" {
                fen = String::from("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
            } else if next == "fen" {
                while next != "moves" {
                    next = args.next().unwrap_or("");
                    if next != "moves" {
                        fen.push_str(next);
                        fen.push_str(" ");
                    }
                }
            } else if next == "moves" {
                while let Some(m) = args.next() {
                    moves.push(ChessMove::from_str(m).unwrap());
                }
            }
            next = args.next().unwrap_or("");
        }
        self.uci.board = Board::from_str(&fen).unwrap();
        for m in moves {
            self.uci.board = self.uci.board.make_move_new(m);
        }
    }

    fn go(&mut self, mut args: std::str::SplitWhitespace) {
        self.uci.time = 0;
        self.uci.inc = 0;
        self.uci.movestogo = 0;
        self.uci.depth = 0;
        self.uci.nodes = 0;
        self.uci.mate = 0;
        self.uci.movetime = 0;
        self.uci.infinite = false;
        self.uci.searching = true;
        let mut next = args.next().unwrap_or("");
        while next != "" {
            if next == "wtime" {
                self.uci.time = args.next().unwrap_or("0").parse().unwrap_or(0);
            } else if next == "btime" {
                self.uci.time = args.next().unwrap_or("0").parse().unwrap_or(0);
            } else if next == "winc" {
                self.uci.inc = args.next().unwrap_or("0").parse().unwrap_or(0);
            } else if next == "binc" {
                self.uci.inc = args.next().unwrap_or("0").parse().unwrap_or(0);
            } else if next == "movestogo" {
                self.uci.movestogo = args.next().unwrap_or("0").parse().unwrap_or(0);
            } else if next == "depth" {
                self.uci.depth = args.next().unwrap_or("0").parse().unwrap_or(0);
            } else if next == "nodes" {
                self.uci.nodes = args.next().unwrap_or("0").parse().unwrap_or(0);
            } else if next == "mate" {
                self.uci.mate = args.next().unwrap_or("0").parse().unwrap_or(0);
            } else if next == "movetime" {
                self.uci.movetime = args.next().unwrap_or("0").parse().unwrap_or(0);
            } else if next == "infinite" {
                self.uci.infinite = true;
            }
            next = args.next().unwrap_or("");
        }
        if self.uci.infinite {
            self.search();
        } else if self.uci.movetime > 0 {
            self.search();
        } else if self.uci.depth > 0 {
            self.search();
        } else if self.uci.nodes > 0 {
            self.search();
        } else if self.uci.time > 0 {
            self.search();
        } else {
            self.search();
        }
    }

    // fn search(&mut self) {
    //     if self.uci.movetime > 0 {
    //         let (best_move, score) = best_move_with_time(&self.uci.board, self.uci.movetime / 100);
    //         println!("bestmove {}", best_move);
    //         return;
    //     }
    //     if self.uci.depth > 0 {
    //         let (best_move, score) = best_move(&self.uci.board, self.uci.depth as u8);
    //         println!("bestmove {}", best_move);
    //         return;
    //     }
    //     let (best_move, score) = best_move(&self.uci.board, 4);
    //     println!("bestmove {}", best_move);
    // }

    fn search(&mut self) {
        let (best_move, score) = best_move(&self.uci.board, 4);
        println!("bestmove {}", best_move);
    }

    fn stop(&mut self) {
        self.uci.searching = false;
    }

    fn quit(&mut self) {
        self.uci.searching = false;
    }

}

pub(crate) fn main() {
    let mut listener = Listener::new();
    let mut line = String::new();
    while let Ok(_) = std::io::stdin().read_line(&mut line) {
        listener.handle(&line);
        line.clear();
    }
}