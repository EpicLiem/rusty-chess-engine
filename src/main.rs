use std::str::FromStr;
use std;
use chess::{Board, MoveGen, BitBoard, Piece, Square, Color, ChessMove};
mod weights;
use weights::*;

const ALL_SQUARES: [Square; 64] = [Square::A1, Square::A2, Square::A3, Square::A4, Square::A5, Square::A6, Square::A7, Square::A8,
                                   Square::B1, Square::B2, Square::B3, Square::B4, Square::B5, Square::B6, Square::B7, Square::B8,
                                   Square::C1, Square::C2, Square::C3, Square::C4, Square::C5, Square::C6, Square::C7, Square::C8,
                                   Square::D1, Square::D2, Square::D3, Square::D4, Square::D5, Square::D6, Square::D7, Square::D8,
                                   Square::E1, Square::E2, Square::E3, Square::E4, Square::E5, Square::E6, Square::E7, Square::E8,
                                   Square::F1, Square::F2, Square::F3, Square::F4, Square::F5, Square::F6, Square::F7, Square::F8,
                                   Square::G1, Square::G2, Square::G3, Square::G4, Square::G5, Square::G6, Square::G7, Square::G8,
                                   Square::H1, Square::H2, Square::H3, Square::H4, Square::H5, Square::H6, Square::H7, Square::H8];

const ALPHA: i32 = -10000000;
const BETA: i32 = 10000000;

struct Node {
    board: Board,
    score: i32,
    depth: i32,
    children: Vec<Node>,
}


impl Node {
    fn new(board: Board) -> Node {
        Node {
            board: board,
            score: 0,
            depth: 0,
            children: Vec::new(),
        }
    }
    fn search(&mut self, depth: i32) -> i32 {
        // search using alpha beta pruning
        let mut early_exit = 0;
        self.depth = depth;
        if depth == 0 || self.board.status() != chess::BoardStatus::Ongoing {
            self.score = evaluation_middlegame(&self.board);
            self.depth = 0;
            return depth;
        }
        let mut alpha = ALPHA;
        let mut beta = BETA;
        if self.board.side_to_move() == Color::White {
            self.score = ALPHA;
            for m in MoveGen::new_legal(&self.board) {
                let mut child = Node::new(self.board.make_move_new(m));
                early_exit = child.search(depth - 1);
                self.depth -= early_exit;
                self.score = self.score.max(child.score);
                self.children.push(child);
                alpha = std::cmp::max(alpha, self.score);
                if beta <= alpha || early_exit != 0 {
                    break;
                }
            }
        } else {
            self.score = BETA;
            for m in MoveGen::new_legal(&self.board) {
                let mut child = Node::new(self.board.make_move_new(m));
                early_exit = child.search(depth - 1);
                self.depth -= early_exit;
                self.score = self.score.min(child.score);
                self.children.push(child);
                beta = std::cmp::min(beta, self.score);
                if beta <= alpha || early_exit != 0 {
                    break;
                }
            }
        }
        return early_exit;
    }
}


fn evaluation_middlegame(board : &Board) -> i32 {
    let pawn_value = 100;
    let knight_value = 300;
    let bishop_value = 300;
    let rook_value = 500;
    let queen_value = 900;
    let king_value = 10000;

    let mut piece_value = 1;
    let mut piece_count = 0;
    let mut score = 0;

    for square in ALL_SQUARES.iter() {
        let piece = board.piece_on(*square);
        let file = square.get_file();
        let rank = square.get_rank();
        match piece {
            Some(Piece::Pawn) => {
                piece_value = pawn_value + PAWN_MG_WT[file.to_index()][rank.to_index()];
                piece_count += 1;
            },
            Some(Piece::Knight) => {
                piece_value = knight_value + KNIGHT_MG_WT[file.to_index()][rank.to_index()];
                piece_count += 1;
            },
            Some(Piece::Bishop) => {
                piece_value = bishop_value + BISHOP_MG_WT[file.to_index()][rank.to_index()];
                piece_count += 1;
            },
            Some(Piece::Rook) => {
                piece_value = rook_value + ROOK_MG_WT[file.to_index()][rank.to_index()];
                piece_count += 1;
            },
            Some(Piece::Queen) => {
                piece_value = queen_value + QUEEN_MG_WT[file.to_index()][rank.to_index()];
                piece_count += 1;
            },
            Some(Piece::King) => {
                piece_value = king_value + KING_MG_WT[file.to_index()][rank.to_index()];
                piece_count += 1;
            },
            None => (),
        }

            if board.side_to_move() == Color::White {
                score += piece_value;
                if board.status() == chess::BoardStatus::Checkmate {
                    score = 10000000;
                }
            } else {
                score -= piece_value;
                if board.status() == chess::BoardStatus::Checkmate {
                    score = 10000000;
                }
            }

        }
    score as i32
    }
//
// fn evaluation_function(&board : Board) -> i32 {
//     // TODO: implement evaluation function that returns a score for the board using middlegame and endgame evaluation functions
// }




fn best_move(board : &Board) -> ChessMove {
    // high level function that returns the best move for the current board
    if board.status() != chess::BoardStatus::Ongoing {
        println!("Game is over!");
        return ChessMove::new(Square::A1, Square::A1, None);
    }
    let mut moves = MoveGen::new_legal(&board);
    let mut best_move = ChessMove::new(Square::A1, Square::A1, None);
    let mut best_node = Node::new(board.clone());
    for m in moves {
        if board.side_to_move() == Color::White {
            let mut best_score = -100000;
            let tboard = board.make_move_new(m);
            let mut node = Node::new(tboard);
            node.search(5);
            println!("Move: {}, Score: {}, Depth: {}", m.to_string(), node.score, node.depth);
            if node.score == 10000000 && node.depth == 0 {
                return m;
            }
            if best_score == 100000 && node.score == 100000 && node.depth < best_node.depth {
                best_score = node.score;
                best_node = node;
                best_move = m;
            }
            else if node.score > best_score {
                best_score = node.score;
                best_node = node;
                best_move = m;
            }
        } else {
            let mut best_score = 100000;
            let tboard = board.make_move_new(m);
            let mut node = Node::new(tboard);
            node.search(5);
            if node.score < best_score {
                best_score = node.score;
                best_node = node;
                best_move = m;
            }
        }
    }
    best_move
}

fn main() {
    let board = Board::from_str("8/8/8/4Q3/2K5/8/8/1k6 w - - 4 3").expect("Invalid FEN");
    let mut node = Node::new(board);
    node.search(5);
    println!("{:?}, {:?}, {:?}", evaluation_middlegame(&board), node.score , best_move(&board).to_string());
}


