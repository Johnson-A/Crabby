use std::fmt;
use std::cmp;
use std::ascii::AsciiExt;
use piece::*;
use util::*;

pub type Squares = [Square; 64];

pub fn gen_bitboards(sqs: &Squares) -> (BitBoard, BitBoard) {
    let mut w: BitBoard = Default::default();
    let mut b: BitBoard = Default::default();

    for i in 0..64 {
        match sqs[i] {
            Square::Piece(pt, col) => {
                let bb = if col == Color::White { &mut w } else { &mut b };
                match pt {
                    PieceType::Pawn   => bb.pawn   |= 1 << i,
                    PieceType::Knight => bb.knight |= 1 << i,
                    PieceType::Bishop => bb.bishop |= 1 << i,
                    PieceType::Rook   => bb.rook   |= 1 << i,
                    PieceType::Queen  => bb.queen  |= 1 << i,
                    PieceType::King   => bb.king   |= 1 << i
                };
            },
            Square::Empty => continue
        }
    }

    w.pieces = w.pawn | w.knight | w.bishop | w.rook | w.queen | w.king;
    b.pieces = b.pawn | b.knight | b.bishop | b.rook | b.queen | b.king;
    (w, b)
}

#[derive(Copy, Clone, Debug)]
pub struct Move { data: u32 }

impl Move {
    pub fn new(from: u32, to: u32, flags: u32) -> Move {
        let d = from | to << 6 | flags << 12;
        Move { data: d }
    }

    pub fn from(&self)  -> u32 { self.data & 0x3F }
    pub fn to(&self)    -> u32 { (self.data >> 6) & 0x3F }
    pub fn flags(&self) -> u32 { self.data >> 12 }
}

pub fn move_to_str(mv: &Move) -> String {
    let (from, to) = (mv.from() as u8, mv.to() as u8);
    let (sr, sc) = (from / 8, from % 8);
    let (dr, dc) = (to / 8, to % 8);
    let (sr_char, sc_char) = ((sr + b'1') as char, (sc + b'a') as char);
    let (dr_char, dc_char) = ((dr + b'1') as char, (dc + b'a') as char);
    let chars = vec![sc_char, sr_char, dc_char, dr_char];
    chars.into_iter().collect::<String>()
}

pub fn get_line_attacks(piece: u64, occ: u64, mask: u64) -> u64 {
    let pot_blockers = occ & mask;
    let forward = pot_blockers - 2*piece;
    let rev = reverse(reverse(pot_blockers) - 2*reverse(piece));
    (forward ^ rev) & mask
}

#[derive(Copy)]
pub struct Board {
    pub w: BitBoard,
    pub b: BitBoard,
    pub sqs: Squares,
    pub move_num: u32,
    pub w_castle: bool,
    pub b_castle: bool,
    pub en_passant: u64
}

impl Clone for Board { fn clone(&self) -> Self { *self } }

impl Board {
    pub fn add_moves(&self, moves: &mut Vec<Move>, mut targets: u64, diff: i32) {
        while targets != 0 {
            let to = bit_pop_pos(&mut targets);
            let from = ((to as i32) - diff) as u32;
            // let capture = board
            moves.push(Move::new(from, to, 0));
        }
    }

    pub fn for_all_pieces(&self, mut pieces: u64, moves: &mut Vec<Move>,
                        attacks: &Fn(u32, u64) -> u64) {
        while pieces != 0 {
            let piece = bit_pop(&mut pieces);
            let from = piece.trailing_zeros();

            let attacks = attacks(from, piece);
            self.add_moves_from(moves, attacks, from);
        }
    }

    pub fn add_moves_from(&self, moves: &mut Vec<Move>, mut targets: u64, from: u32) {
        while targets != 0 {
            let to = bit_pop_pos(&mut targets);
            moves.push(Move::new(from, to, 0));
        }
    }

    pub fn make_move(&mut self, mv: Move) {
        let (src, dest) = (mv.from() as usize, mv.to() as usize);
        self.sqs[dest] = self.sqs[src];
        self.sqs[src] = Square::Empty;
        let (w, b) = gen_bitboards(&self.sqs);
        self.w = w;
        self.b = b;
        self.move_num += 1;
    }

    pub fn make_promotion(&mut self, mv: Move, prom: Square) {
        let (src, dest) = (mv.from() as usize, mv.to() as usize);
        self.sqs[dest] = prom;
        self.sqs[src] = Square::Empty;
        let (w, b) = gen_bitboards(&self.sqs);
        self.w = w;
        self.b = b;
        self.move_num += 1;
    }

    pub fn make_str_move(&mut self, mv: &str) {
        let moves: Vec<char> = mv.chars().collect();
        match moves.as_slice() {
            // [sc, sr, dc, dr, promotion..] => {
            [sc, sr, dc, dr] => {
                self.make_move(Move::new(to_pos(sc, sr), to_pos(dc, dr), 0));
            },
            [sc, sr, dc, dr, promotion] => {
                let prom_piece = to_piece(if self.move_num % 2 == 1 {promotion.to_ascii_uppercase()} else {promotion});
                self.make_promotion(Move::new(to_pos(sc, sr), to_pos(dc, dr), 0), prom_piece); // TODO:
            }
            _ => () // malformed move
        }
    }

    pub fn get_pseudo_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::with_capacity(64);

        let is_white = self.move_num % 2 == 1;
        let (us, opp) = if is_white { (&self.w, &self.b) } else { (&self.b, &self.w) };

        let occ = us.pieces | opp.pieces;

        // Consider out of bounds pawn promotion
        // Implement pawn promotion!!!! king won't know it's checked
        // And the move number won't change when accepting a promotion capture move
        // Make move not copyable
        if is_white {
            let mut pushes = (us.pawn << 8) & !occ;
            let double_pushes = ((pushes & ROW_3) << 8) & !occ;
            let left_attacks = (us.pawn << 7) & (opp.pieces | self.en_passant) & !FILE_H;
            let right_attacks = (us.pawn << 9) & (opp.pieces | self.en_passant) & !FILE_A;
            let promotions = pushes & ROW_8;
            pushes &= !ROW_8;

            self.add_moves(&mut moves, pushes, 8);
            self.add_moves(&mut moves, double_pushes, 16);
            self.add_moves(&mut moves, left_attacks, 7);
            self.add_moves(&mut moves, right_attacks, 9);
            self.add_moves(&mut moves, promotions, 8);
        } else {
            let mut pushes = (us.pawn >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (us.pawn >> 7) & (opp.pieces | self.en_passant) & !FILE_A;
            let right_attacks = (us.pawn >> 9) & (opp.pieces | self.en_passant) & !FILE_H;
            let promotions = pushes & ROW_1;
            pushes &= !ROW_1;

            self.add_moves(&mut moves, pushes, -8);
            self.add_moves(&mut moves, double_pushes, -16);
            self.add_moves(&mut moves, left_attacks, -7);
            self.add_moves(&mut moves, right_attacks, -9);
            self.add_moves(&mut moves, promotions, -8);
        }

        self.for_all_pieces(us.queen, &mut moves, &|from, piece| -> u64 {
                (get_line_attacks(piece, occ, file(from)) |
                 get_line_attacks(piece, occ, row(from))  |
                 get_line_attacks(piece, occ, diag(from)) |
                 get_line_attacks(piece, occ, a_diag(from))) & !us.pieces
            });

        self.for_all_pieces(us.rook, &mut moves, &|from, piece| -> u64 {
                (get_line_attacks(piece, occ, file(from)) |
                 get_line_attacks(piece, occ, row(from))) & !us.pieces
            });

        self.for_all_pieces(us.bishop, &mut moves, &|from, piece| -> u64 {
                (get_line_attacks(piece, occ, diag(from)) |
                 get_line_attacks(piece, occ, a_diag(from))) & !us.pieces
            });

        self.for_all_pieces(us.knight, &mut moves, &|from, piece| -> u64 {
                KNIGHT_MAP[from as usize] & !us.pieces
            });

        self.for_all_pieces(us.king, &mut moves, &|from, piece| -> u64 {
                KING_MAP[from as usize] & !us.pieces
            });

        moves
    }

    pub fn to_move(&self) -> &BitBoard {
        if self.move_num % 2 == 1 { &self.w } else { &self.b }
    }

    pub fn prev_move(&self) -> &BitBoard {
        if self.move_num % 2 == 1 { &self.b } else { &self.w }
    }

    pub fn get_moves(&mut self) -> Vec<Move> {
        let pseudo_legal_moves = self.get_pseudo_moves();
        let mut legal_moves = Vec::with_capacity(pseudo_legal_moves.len());

        for mv in pseudo_legal_moves.into_iter() {
            let mut new_board = self.clone();
            new_board.make_move(mv);
            let king_pos = new_board.prev_move().king.trailing_zeros(); // This will be the original player's king
            let mut king_is_attacked = false;

            for opp_mv in new_board.get_pseudo_moves() {
                if opp_mv.to() == king_pos {
                    king_is_attacked = true;
                    break;
                }
            }
            if !king_is_attacked { legal_moves.push(mv) };
        }
        legal_moves
    }

    pub fn evaluate(&self) -> f32 {
        let opp = self.prev_move();
        let us = self.to_move(); // Node player

        (us.pawn.count_ones() as f32  * 1.0)   +
        (us.knight.count_ones() as f32 * 3.0)  +
        (us.bishop.count_ones() as f32 * 3.0)  +
        (us.rook.count_ones() as f32 * 5.0)    +
        (us.queen.count_ones() as f32 * 9.0)   -
        (opp.pawn.count_ones() as f32 * 1.0)   -
        (opp.knight.count_ones() as f32 * 3.0) -
        (opp.bishop.count_ones() as f32 * 3.0) -
        (opp.rook.count_ones() as f32 * 5.0)   -
        (opp.queen.count_ones() as f32 * 9.0)
    }

    pub fn best_move(&mut self) -> Move {
        let mut best_score = 1000.0;
        let mut best_move = Move::new(0,0,0);

        for mv in self.get_moves() {
            let mut new_board = self.clone();
            new_board.make_move(mv);
            println!("Searching \n{}", new_board);
            let score = new_board.negamax(4);
            println!("Found value {}", score);

            if score < best_score {
                best_move = mv;
                best_score = score;
            }
        }
        println!("\nbest score {}", best_score);
        best_move
    }

    pub fn negamax(&mut self, depth: u32) -> f32 {
        if depth == 0 { return self.evaluate() }
        let mut best = -1000.0;
        let moves = self.get_moves();
        if moves.len() == 0 { return -best }
        // println!("\n{}\n {}", self, depth);
        for mv in moves.into_iter() {
            // println!("Counter {}", move_to_str(&mv));
            let mut new_board = self.clone();
            new_board.make_move(mv);
            let score = -new_board.negamax(depth - 1);
            // println!("With value {}", score);

            if score > best { best = score; }
        }
        best
    }

    pub fn new(fen_board: &str) -> Board {
        let reversed_rows = fen_board.split('/').rev(); // fen is read from top rank
        let mut sqs = [Square::Empty; 64];

        for (r, row) in reversed_rows.enumerate() {
            let mut offset = 0;
            for (c, ch) in row.chars().enumerate() {
                if !ch.is_numeric() {
                    sqs[r*8 + c+offset] = to_piece(ch);
                } else {
                    offset += (ch as u8 - b'0') as usize;
                }
            }
        }
        let (w, b) = gen_bitboards(&sqs);

        Board { w: w, b: b, sqs: sqs, move_num: 1, w_castle: true, b_castle: true, en_passant: 0 }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut characters = Vec::with_capacity(64);

        for (i, sq) in self.sqs.iter().enumerate() {
            characters.push(to_char(sq));
            if (i+1) % 8 == 0 { characters.push('\n') }
        }
        let output = characters.iter().cloned().collect::<String>();
        // write!(f, "--------\n{}--------\n\
        //           Move # {:?}\n\
        //           wcas {} bcas {}\nen passant {}",
        //           output, self.move_num,
        //           self.w_castle, self.b_castle, self.en_passant)
        write!(f, "--------\n{}--------\nmove # {}\n", output, self.move_num)
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct BitBoard {
    pub pawn: u64,
    pub knight: u64,
    pub bishop: u64,
    pub rook: u64,
    pub queen: u64,
    pub king: u64,
    pub pieces: u64
}
