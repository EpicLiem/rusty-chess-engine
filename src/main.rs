use std::str::FromStr;
use std;
use chess::{Board, MoveGen, BitBoard, Piece, Square, Color, ChessMove};

const ALL_SQUARES: [Square; 64] = [Square::A1, Square::A2, Square::A3, Square::A4, Square::A5, Square::A6, Square::A7, Square::A8,
                                   Square::B1, Square::B2, Square::B3, Square::B4, Square::B5, Square::B6, Square::B7, Square::B8,
                                   Square::C1, Square::C2, Square::C3, Square::C4, Square::C5, Square::C6, Square::C7, Square::C8,
                                   Square::D1, Square::D2, Square::D3, Square::D4, Square::D5, Square::D6, Square::D7, Square::D8,
                                   Square::E1, Square::E2, Square::E3, Square::E4, Square::E5, Square::E6, Square::E7, Square::E8,
                                   Square::F1, Square::F2, Square::F3, Square::F4, Square::F5, Square::F6, Square::F7, Square::F8,
                                   Square::G1, Square::G2, Square::G3, Square::G4, Square::G5, Square::G6, Square::G7, Square::G8,
                                   Square::H1, Square::H2, Square::H3, Square::H4, Square::H5, Square::H6, Square::H7, Square::H8];

const ALPHA: i32 = -5000000;
const BETA: i32 = 5000000;

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
        self.depth = depth;
        if depth == 0 || self.board.status() != chess::BoardStatus::Ongoing {
            self.score = evaluation_middlegame(&self.board);
            return depth;
        }
        let mut alpha = ALPHA;
        let mut beta = BETA;
        if self.board.side_to_move() == Color::White {
            self.score = ALPHA;
            for m in MoveGen::new_legal(&self.board) {
                let mut child = Node::new(self.board.make_move_new(m));
                let early_exit = child.search(depth - 1);
                self.depth -= early_exit;
                self.score = self.score.max(child.score);
                self.children.push(child);
                alpha = std::cmp::max(alpha, self.score);
                if beta <= alpha {
                    break;
                }
            }
        } else {
            self.score = BETA;
            for m in MoveGen::new_legal(&self.board) {
                let mut child = Node::new(self.board.make_move_new(m));
                let early_exit = child.search(depth - 1);
                self.depth -= early_exit;
                self.score = self.score.min(child.score);
                self.children.push(child);
                beta = std::cmp::min(beta, self.score);
                if beta <= alpha {
                    break;
                }
            }
        }
        return 0;
    }
}


fn evaluation_middlegame(board : &Board) -> i32 {
    let mut score = 0;
    let mut pawn_value = 100;
    let mut knight_value = 300;
    let mut bishop_value = 330;
    let mut rook_value = 500;
    let mut queen_value = 900;
    let mut king_value = 20000;

    let mut piece_value = 1;
    let mut piece_count = 0;

    for square in ALL_SQUARES.iter() {
        let piece = board.piece_on(*square);
        let file = square.get_file();
        let rank = square.get_rank();
        match piece {
            Some(Piece::Pawn) => {
                piece_value = pawn_value;
                // center control
                if file.to_index() == 3 || file.to_index() == 4 {
                    piece_value += 10;
                }
                // doubled pawns
                if board.piece_on(Square::make_square(rank.up() , file)) == Some(Piece::Pawn) {
                    piece_value -= 20;
                }
                // isolated pawns
                if board.piece_on(Square::make_square(rank, file.right())) != Some(Piece::Pawn) && board.piece_on(Square::make_square(rank, file.left())) != Some(Piece::Pawn) {
                    piece_value -= 10;
                }
                // backward pawns
                if board.piece_on(Square::make_square(rank.up(), file.right())) == Some(Piece::Pawn) || board.piece_on(Square::make_square(rank.up() , file.left())) == Some(Piece::Pawn) {
                    piece_value -= 10;
                }
                // passed pawns
                if board.piece_on(Square::make_square(rank.down(), file.right())) != Some(Piece::Pawn) && board.piece_on(Square::make_square(rank.down(), file.right())) != Some(Piece::Pawn) {
                    piece_value += 20;
                }
                piece_count += 1;
            },
            Some(Piece::Knight) => {
                piece_value = knight_value;

                // amount of moves
                piece_value += chess::get_knight_moves(*square).popcnt();

                piece_count += 1;
            },
            Some(Piece::Bishop) => {
                piece_value = bishop_value;

                // amount of moves
                piece_value += chess::get_bishop_moves(*square, *board.combined()).popcnt();

                piece_count += 1;
            },
            Some(Piece::Rook) => {
                piece_value = rook_value;

                // file control
                if file.to_index() == 0 || file.to_index() == 7 {
                    piece_value += 10;
                }
                // amount of moves
                piece_value += chess::get_rook_moves(*square, *board.combined()).popcnt();

                // rook on open file
                if board.piece_on(Square::make_square(rank.up(), file)) != Some(Piece::Pawn) && board.piece_on(Square::make_square(rank.down(), file)) != Some(Piece::Pawn) {
                    piece_value += 10;
                }
                piece_count += 1;
            },
            Some(Piece::Queen) => {
                piece_value = queen_value;

                // queen on open file
                if board.piece_on(Square::make_square(rank.up(), file)) != Some(Piece::Pawn) && board.piece_on(Square::make_square(rank.down(), file)) != Some(Piece::Pawn) {
                    piece_value += 10;
                }

                piece_count += 1;
            },
            Some(Piece::King) => {
                piece_value = king_value;
                // safety
                piece_value -= chess::get_king_moves(*square).popcnt();

                piece_count += 1;
            },
            None => {
                piece_value = 0;
                piece_count = 0;
            },
        }
        if board.color_on(*square) == Some(Color::White) {
            score += piece_value as i32;
        } else {
            score -= piece_value as i32;
        }
    }
    if board.status() == chess::BoardStatus::Checkmate {
        if board.side_to_move() == Color::White {
            score = -100000;
        } else {
            score = 100000;
        }
    }
    if board.status() == chess::BoardStatus::Stalemate {
        score = 0;
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
            if node.score > best_score {
                best_score = node.score;
                best_node = node;
                best_move = m;
            }
            else if best_score == 100000 && node.score == 100000 && node.depth < best_node.depth {
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
    let board = Board::from_str("8/8/8/8/1K6/2Q5/8/1k6 w - - 0 1").expect("Invalid FEN");
    let mut node = Node::new(board);
    node.search(5);
    println!("{:?}, {:?}, {:?}", evaluation_middlegame(&board), node.score , best_move(&board).to_string());
}


