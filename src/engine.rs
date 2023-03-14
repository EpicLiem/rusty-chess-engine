use chess::{BitBoard, Board, BoardStatus, ChessMove, Color, MoveGen, Piece, Square};
use std;
use std::cmp::Ordering;
use std::str::FromStr;
use std::time::{Duration, Instant};
use std::sync::mpsc;

use crate::weights::*;

const ALPHA: i32 = i32::MIN;
const BETA: i32 = i32::MAX;

const PAWN_MG_VAL: i32 = 100;
const KNIGHT_MG_VAL: i32 = 320;
const BISHOP_MG_VAL: i32 = 330;
const ROOK_MG_VAL: i32 = 500;
const QUEEN_MG_VAL: i32 = 900;
const KING_MG_VAL: i32 = 20000;

const BISHOP_PAIR_MG_VAL: i32 = 50;
const ROOK_ON_OPEN_FILE_MG_VAL: i32 = 10;
const ROOK_ON_SEMI_OPEN_FILE_MG_VAL: i32 = 5;
const ROOK_ON_7TH_MG_VAL: i32 = 20;
const ROOK_ON_8TH_MG_VAL: i32 = 30;
const QUEEN_ON_7TH_MG_VAL: i32 = 10;
const QUEEN_ON_8TH_MG_VAL: i32 = 20;
const PAWN_ON_7TH_MG_VAL: i32 = 10;
const ISOLATED_PAWN_MG_VAL: i32 = -10;
const DOUBLED_PAWN_MG_VAL: i32 = -10;
const BACKWARD_PAWN_MG_VAL: i32 = -5;
const PASSED_PAWN_MG_VAL: i32 = 10;
const CENTER_CONTROL_MG_VAL: i32 = 5;
const KING_SAFETY_MG_VAL: i32 = 5;

const FILES: [BitBoard; 8] = [
    BitBoard(0x0101010101010101),
    BitBoard(0x0202020202020202),
    BitBoard(0x0404040404040404),
    BitBoard(0x0808080808080808),
    BitBoard(0x1010101010101010),
    BitBoard(0x2020202020202020),
    BitBoard(0x4040404040404040),
    BitBoard(0x8080808080808080),
];
const RANKS: [BitBoard; 8] = [
    BitBoard(0x00000000000000FF),
    BitBoard(0x000000000000FF00),
    BitBoard(0x0000000000FF0000),
    BitBoard(0x00000000FF000000),
    BitBoard(0x000000FF00000000),
    BitBoard(0x0000FF0000000000),
    BitBoard(0x00FF000000000000),
    BitBoard(0xFF00000000000000),
];
const CENTER: BitBoard = BitBoard(0x0000001818000000);

#[derive(Clone, Copy, Debug, Default)]
pub struct Score {
    pub(crate) eval: i32,
    mate: bool,
    ply: i32,
    color: bool, // should only be used when mate is true; true if white is winning
}

pub(crate) struct Searcher {
    board: Board,
    pub(crate) depth: u8,
    pub(crate) best_move: ChessMove,
    best_score: Score,
    children: Vec<Searcher>,
}

impl Searcher {
    pub(crate) fn new(board: &Board, depth: u8) -> Searcher {
        Searcher {
            board: board.clone(),
            depth: depth,
            best_move: Default::default(),
            best_score: Score {
                eval: 0,
                mate: false,
                ply: 0,
                color: false,
            },
            children: Vec::new(),
        }
    }
    pub(crate) fn alpha_beta(&mut self, alpha: i32, beta: i32) -> Score {
        if self.depth == 0 || self.board.status() != BoardStatus::Ongoing {
            return evaluation_middlegame(&self.board);
        }
        let mut alpha = Score {
            eval: alpha,
            mate: false,
            ply: 0,
            color: false,
        };
        let mut beta = Score {
            eval: beta,
            mate: false,
            ply: 0,
            color: false,
        };
        let mut best_score = if self.board.side_to_move() == Color::White {
            Score {
                eval: ALPHA,
                mate: false,
                ply: 0,
                color: false,
            }
        } else {
            Score {
                eval: BETA,
                mate: false,
                ply: 0,
                color: false,
            }
        };
        let mut best_move = Default::default();
        let mut children = Vec::new();
        if self.board.side_to_move() == Color::White {
            for m in MoveGen::new_legal(&self.board) {
                let mut child = Searcher::new(&self.board.make_move_new(m), self.depth - 1);
                let score = child.alpha_beta(beta.eval, alpha.eval);
                children.push(child);
                if score > best_score {
                    best_score = score;
                    best_move = m;
                }
                if score > alpha {
                    assert_eq!(score, best_score);
                    alpha = score;
                }
                if alpha >= beta {
                    break;
                }
            }
        } else {
            for m in MoveGen::new_legal(&self.board) {
                let mut child = Searcher::new(&self.board.make_move_new(m), self.depth - 1);
                let score = child.alpha_beta(beta.eval, alpha.eval);
                children.push(child);
                if score < best_score {
                    best_score = score;
                    best_move = m;
                }
                if score < beta {
                    beta = score;
                }
                if alpha >= beta {
                    best_score = score;
                    break;
                }
            }
        }
        self.best_move = best_move;
        self.best_score = best_score;
        self.children = children;
        if best_score.mate {
            if best_score.color {
                best_score.ply += 1;
            } else {
                best_score.ply -= 1;
            }
        }
        best_score
    }
    pub(crate) fn alpha_beta_with_time(&mut self, time: Duration) -> Score {
        let start = Instant::now();
        let mut depth = 1;
        let mut best_score = Score {
            eval: 0,
            mate: false,
            ply: 0,
            color: false,
        };
        while start.elapsed() < time {
            let estimated_time = time.as_secs_f64() / 2.5_f64.powi(depth as i32);
            if estimated_time < 0.1 {
                break;
            }
            best_score = self.alpha_beta(ALPHA, BETA);
            self.depth += 1;
        }
        best_score
    }
    pub(crate) fn alpha_beta_until_stopped(&mut self, reciever: mpsc::Receiver<bool>) -> Score {
        loop {
            if reciever.try_recv() == Ok(true) {
                break;
            }
            self.alpha_beta(ALPHA, BETA);
            self.depth += 1;
        }
        return self.best_score;
    }
}

impl Score {
    fn new_eval(eval: i32) -> Score {
        Score {
            eval: eval,
            mate: false,
            ply: 0,
            color: false,
        }
    }
    fn new_mate(ply: i32, color: bool) -> Score {
        Score {
            eval: 0,
            mate: true,
            ply: ply,
            color: color,
        }
    }
}

impl PartialEq<Self> for Score {
    fn eq(&self, other: &Self) -> bool {
        if self.mate == other.mate {
            if self.mate {
                self.ply == other.ply && self.color == other.color // only matters if mate is true
            } else {
                self.eval == other.eval // only matters if mate is false
            }
        } else {
            false // mate isn't equal to non-mate
        }
    }
}

impl PartialOrd<Self> for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.mate {
            if other.mate {
                if self.color == other.color {
                    return self.ply.partial_cmp(&other.ply);
                } else if self.color {
                    return Some(Ordering::Greater);
                } else {
                    return Some(Ordering::Less);
                }
            } else {
                return Some(Ordering::Greater);
            }
        } else {
            return self.eval.partial_cmp(&other.eval);
        }
    }
}

pub(crate) fn evaluation_middlegame(board: &Board) -> Score {
    let mut evaluation = 0;
    if board.status() == BoardStatus::Ongoing {
        // Bitboards
        let pawns = board.pieces(Piece::Pawn);
        let knights = board.pieces(Piece::Knight);
        let bishops = board.pieces(Piece::Bishop);
        let rooks = board.pieces(Piece::Rook);
        let queens = board.pieces(Piece::Queen);
        let kings = board.pieces(Piece::King);
        let white = board.color_combined(Color::White);
        let black = board.color_combined(Color::Black);
        let white_pawns = pawns & white;
        let black_pawns = pawns & black;
        let white_knights = knights & white;
        let black_knights = knights & black;
        let white_bishops = bishops & white;
        let black_bishops = bishops & black;
        let white_rooks = rooks & white;
        let black_rooks = rooks & black;
        let white_queens = queens & white;
        let black_queens = queens & black;
        let white_kings = kings & white;
        let black_kings = kings & black;

        // Piece counts
        let white_pawn_count = white_pawns.popcnt();
        let black_pawn_count = black_pawns.popcnt();
        let white_knight_count = white_knights.popcnt();
        let black_knight_count = black_knights.popcnt();
        let white_bishop_count = white_bishops.popcnt();
        let black_bishop_count = black_bishops.popcnt();
        let white_rook_count = white_rooks.popcnt();
        let black_rook_count = black_rooks.popcnt();
        let white_queen_count = white_queens.popcnt();
        let black_queen_count = black_queens.popcnt();
        let white_king_count = white_kings.popcnt();
        let black_king_count = black_kings.popcnt();

        // Total pieces bitboards
        let all_pieces = board.combined();
        let white_pieces = board.color_combined(Color::White);
        let black_pieces = board.color_combined(Color::Black);

        // Total piece counts
        let all_pieces_count = all_pieces.popcnt();
        let white_pieces_count = white_pieces.popcnt();
        let black_pieces_count = black_pieces.popcnt();

        // Material
        evaluation += PAWN_MG_VAL * (white_pawn_count as i32 - black_pawn_count as i32);
        evaluation += KNIGHT_MG_VAL * (white_knight_count as i32 - black_knight_count as i32);
        evaluation += BISHOP_MG_VAL * (white_bishop_count as i32 - black_bishop_count as i32);
        evaluation += ROOK_MG_VAL * (white_rook_count as i32 - black_rook_count as i32);
        evaluation += QUEEN_MG_VAL * (white_queen_count as i32 - black_queen_count as i32);
        evaluation += KING_MG_VAL * (white_king_count as i32 - black_king_count as i32);

        // Bishop pairs
        if black_bishop_count == 2 {
            evaluation += BISHOP_PAIR_MG_VAL;
        }

        if white_bishop_count == 2 {
            evaluation -= BISHOP_PAIR_MG_VAL;
        }

        // Rook logic
        if white_rook_count > 0 {
            // Rook on open file
            let white_rooks_files = [
                white_rooks & FILES[0],
                white_rooks & FILES[1],
                white_rooks & FILES[2],
                white_rooks & FILES[3],
                white_rooks & FILES[4],
                white_rooks & FILES[5],
                white_rooks & FILES[6],
                white_rooks & FILES[7],
            ];
            for fileindex in 0..7 {
                let file = white_rooks_files[fileindex];
                if file.popcnt() > 0 {
                    if FILES[fileindex] & white_pawns == BitBoard::new(0)
                        && FILES[fileindex] & black_pawns == BitBoard::new(0)
                    {
                        evaluation += ROOK_ON_OPEN_FILE_MG_VAL;
                    }
                    // Rook on half open file
                    else if FILES[fileindex] & white_pawns == BitBoard::new(0) {
                        evaluation += ROOK_ON_SEMI_OPEN_FILE_MG_VAL;
                    }
                }
            }
            // rook on 7th rank
            if white_rooks & RANKS[6] != BitBoard::new(0) {
                evaluation += ROOK_ON_7TH_MG_VAL;
            }
            // rook on 8th rank
            if white_rooks & RANKS[7] != BitBoard::new(0) {
                evaluation += ROOK_ON_8TH_MG_VAL;
            }
        }

        if black_rook_count > 0 {
            let black_rooks_files = [
                black_rooks & FILES[0],
                black_rooks & FILES[1],
                black_rooks & FILES[2],
                black_rooks & FILES[3],
                black_rooks & FILES[4],
                black_rooks & FILES[5],
                black_rooks & FILES[6],
                black_rooks & FILES[7],
            ];
            for fileindex in 0..7 {
                let file = black_rooks_files[fileindex];
                if file.popcnt() > 0 {
                    if FILES[fileindex] & black_pawns == BitBoard::new(0)
                        && FILES[fileindex] & white_pawns == BitBoard::new(0)
                    {
                        evaluation -= ROOK_ON_OPEN_FILE_MG_VAL;
                    } else if FILES[fileindex] & black_pawns == BitBoard::new(0) {
                        evaluation -= ROOK_ON_SEMI_OPEN_FILE_MG_VAL;
                    }
                }
            }
            // rook on 7th rank
            if black_rooks & RANKS[1] != BitBoard::new(0) {
                evaluation -= ROOK_ON_7TH_MG_VAL;
            }
            // rook on 8th rank
            if black_rooks & RANKS[0] != BitBoard::new(0) {
                evaluation -= ROOK_ON_8TH_MG_VAL;
            }
        }

        // Pawn logic
        if white_pawn_count > 0 {
            // Isolated pawns
            let white_pawns_files = [
                white_pawns & FILES[0],
                white_pawns & FILES[1],
                white_pawns & FILES[2],
                white_pawns & FILES[3],
                white_pawns & FILES[4],
                white_pawns & FILES[5],
                white_pawns & FILES[6],
                white_pawns & FILES[7],
            ];
            let white_pawns_ranks = [
                white_pawns & RANKS[0],
                white_pawns & RANKS[1],
                white_pawns & RANKS[2],
                white_pawns & RANKS[3],
                white_pawns & RANKS[4],
                white_pawns & RANKS[5],
                white_pawns & RANKS[6],
                white_pawns & RANKS[7],
            ];
            for fileindex in 0..7 {
                let file = white_pawns_files[fileindex];
                if file.popcnt() > 0 {
                    if FILES[fileindex] & white_pawns == BitBoard::new(0)
                        && FILES[fileindex] & black_pawns == BitBoard::new(0)
                    {
                        evaluation += ISOLATED_PAWN_MG_VAL;
                    }
                }
            }
            // Doubled pawns
            for rankindex in 0..7 {
                let rank = white_pawns_ranks[rankindex];
                if rank.popcnt() > 1 {
                    evaluation += DOUBLED_PAWN_MG_VAL;
                }
            }
            // Passed pawns
            for rankindex in 0..7 {
                let rank = white_pawns_ranks[rankindex];
                if rank.popcnt() > 0 {
                    if RANKS[rankindex] & black_pawns == BitBoard::new(0) {
                        evaluation += PASSED_PAWN_MG_VAL;
                    }
                }
            }
            // Pawn on 7th rank
            if white_pawns & RANKS[6] != BitBoard::new(0) {
                evaluation += PAWN_ON_7TH_MG_VAL;
            }
            // Central Control
            evaluation += (white_pawns & CENTER).popcnt() as i32 * CENTER_CONTROL_MG_VAL;
        }
        if black_pawn_count > 0 {
            // Isolated pawns
            let black_pawns_files = [
                black_pawns & FILES[0],
                black_pawns & FILES[1],
                black_pawns & FILES[2],
                black_pawns & FILES[3],
                black_pawns & FILES[4],
                black_pawns & FILES[5],
                black_pawns & FILES[6],
                black_pawns & FILES[7],
            ];
            let black_pawns_ranks = [
                black_pawns & RANKS[0],
                black_pawns & RANKS[1],
                black_pawns & RANKS[2],
                black_pawns & RANKS[3],
                black_pawns & RANKS[4],
                black_pawns & RANKS[5],
                black_pawns & RANKS[6],
                black_pawns & RANKS[7],
            ];
            for fileindex in 0..7 {
                let file = black_pawns_files[fileindex];
                if file.popcnt() > 0 {
                    if FILES[fileindex] & black_pawns == BitBoard::new(0)
                        && FILES[fileindex] & white_pawns == BitBoard::new(0)
                    {
                        evaluation -= ISOLATED_PAWN_MG_VAL;
                    }
                }
            }
            // Doubled pawns
            for rankindex in 0..7 {
                let rank = black_pawns_ranks[rankindex];
                if rank.popcnt() > 1 {
                    evaluation -= DOUBLED_PAWN_MG_VAL;
                }
            }
            // Passed pawns
            for rankindex in 0..7 {
                let rank = black_pawns_ranks[rankindex];
                if rank.popcnt() > 0 {
                    if RANKS[rankindex] & white_pawns == BitBoard::new(0) {
                        evaluation -= PASSED_PAWN_MG_VAL;
                    }
                }
            }
            // Pawn on 7th rank
            if black_pawns & RANKS[1] != BitBoard::new(0) {
                evaluation -= PAWN_ON_7TH_MG_VAL;
            }
            // Central Control
            if black_pawns & CENTER != BitBoard::new(0) {
                evaluation -= CENTER_CONTROL_MG_VAL;
            }
        }
        return Score::new_eval(evaluation);
    } else if board.status() == BoardStatus::Checkmate {
        if board.side_to_move() == Color::White {
            return Score::new_mate(-0, false);
        } else {
            return Score::new_mate(0, true);
        }
    } else if board.status() == BoardStatus::Stalemate {
        evaluation = 0;
    }
    return Score::new_eval(evaluation);
}

pub fn best_move(board: &Board, depth: u8) -> (ChessMove, i32) {
    let mut searcher = Searcher::new(board, depth);
    searcher.alpha_beta(ALPHA, BETA);
    return (searcher.best_move, searcher.best_score.eval);
}

pub fn best_move_with_time(board: &Board, time: u64) -> (ChessMove, i32) {
    let mut searcher = Searcher::new(board, 0);
    searcher.alpha_beta_with_time(Duration::from_millis(time));
    return (searcher.best_move, searcher.best_score.eval);
}

pub fn best_move_infinite_thread(board: &Board, reciever: mpsc::Receiver<bool>) -> (ChessMove, i32) {
    let mut searcher = Searcher::new(board, 1);
    searcher.alpha_beta_until_stopped(reciever);
    return (searcher.best_move.clone(), searcher.best_score.eval.clone());
}