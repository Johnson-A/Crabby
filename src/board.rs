use std::ascii::AsciiExt;
use std::fmt;
use types::*;
use util::*;

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

pub fn add_moves(moves: &mut Vec<Move>, mut targets: u64, diff: i32) {
    while targets != 0 {
        let to = bit_pop_pos(&mut targets);
        let from = ((to as i32) - diff) as u32;
        // let capture = board
        moves.push(Move::new(from, to, 0));
    }
}

pub fn add_moves_from(moves: &mut Vec<Move>, from: u32, mut targets: u64) {
    while targets != 0 {
        let to = bit_pop_pos(&mut targets);
        moves.push(Move::new(from, to, 0));
    }
}

pub fn for_all_pieces(mut pieces: u64, do_work: &mut FnMut(u32, u64)) {
    while pieces != 0 {
        let piece = bit_pop(&mut pieces);
        let from = piece.trailing_zeros();

        do_work(from, piece);
    }
}

impl Board {
    pub fn make_move(&mut self, mv: Move) {
        let (src, dest) = (mv.from() as usize, mv.to() as usize);
        self.sqs[dest] = self.sqs[src];
        self.sqs[src] = Square::Empty;
        let (w, b) = gen_bitboards(&self.sqs);
        self.w = w;
        self.b = b;
        // Board::update(&mut self.w, &mut self.b, mv);
        self.move_num += 1;
    }

    // pub fn update(us: &mut BitBoard, opp: &mut BitBoard, mv: Move) {
    // TODO:
    // }

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

        // Add pawn moves last

        for_all_pieces(us.queen, &mut |from, piece| {
            add_moves_from(&mut moves, from,
                queen_attacks(piece, from, occ) & !us.pieces);
        });

        for_all_pieces(us.rook, &mut |from, piece| {
            add_moves_from(&mut moves, from,
                rook_attacks(piece, from, occ) & !us.pieces);
        });

        for_all_pieces(us.bishop, &mut |from, piece| {
            add_moves_from(&mut moves, from,
                bishop_attacks(piece, from, occ) & !us.pieces);
        });

        for_all_pieces(us.knight, &mut |from, piece| {
            add_moves_from(&mut moves, from, knight_attacks(from) & !us.pieces);
        });

        for_all_pieces(us.king, &mut |from, piece| {
            add_moves_from(&mut moves, from, king_attacks(from) & !us.pieces);
        });

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

            add_moves(&mut moves, pushes, 8);
            add_moves(&mut moves, double_pushes, 16);
            add_moves(&mut moves, left_attacks, 7);
            add_moves(&mut moves, right_attacks, 9);
            add_moves(&mut moves, promotions, 8);
        } else {
            let mut pushes = (us.pawn >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (us.pawn >> 7) & (opp.pieces | self.en_passant) & !FILE_A;
            let right_attacks = (us.pawn >> 9) & (opp.pieces | self.en_passant) & !FILE_H;
            let promotions = pushes & ROW_1;
            pushes &= !ROW_1;

            add_moves(&mut moves, pushes, -8);
            add_moves(&mut moves, double_pushes, -16);
            add_moves(&mut moves, left_attacks, -7);
            add_moves(&mut moves, right_attacks, -9);
            add_moves(&mut moves, promotions, -8);
        }

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

        let mut mobility = 0.0; // minus double pawn pushes
        let mut attacks = 0.0;
        let mut back_rank = 0.0;

        let is_white = self.move_num % 2 == 1;
        let occ = us.pieces | opp.pieces;

        if is_white {
            back_rank -= ((us.pieces ^ (us.king | us.queen)) & ROW_1).count_ones() as f32 * 0.05;
            back_rank += ((opp.pieces ^ (opp.king | opp.queen)) & ROW_8).count_ones() as f32 * 0.05;

            let pushes = (us.pawn << 8) & !occ;
            let double_pushes = ((pushes & ROW_3) << 8) & !occ;
            let left_attacks = (us.pawn << 7) & (opp.pieces | self.en_passant) & !FILE_H;
            let right_attacks = (us.pawn << 9) & (opp.pieces | self.en_passant) & !FILE_A;

            mobility += pushes.count_ones() as f32 * 0.01 +
                        double_pushes.count_ones() as f32 * -0.01;
            attacks +=  left_attacks.count_ones() as f32   * 0.04 +
                        right_attacks.count_ones() as f32  * 0.04;

            let pushes = (opp.pawn >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (opp.pawn >> 7) & (us.pieces | self.en_passant) & !FILE_A;
            let right_attacks = (opp.pawn >> 9) & (us.pieces | self.en_passant) & !FILE_H;

            mobility -= pushes.count_ones() as f32  * 0.01 +
                        double_pushes.count_ones() as f32  * -0.01;
            attacks -=  left_attacks.count_ones() as f32   * 0.04 +
                        right_attacks.count_ones() as f32  * 0.04;
        } else {
            back_rank -= ((us.pieces ^ (us.king | us.queen)) & ROW_8).count_ones() as f32 * 0.05;
            back_rank += ((opp.pieces ^ (opp.king | opp.queen)) & ROW_1).count_ones() as f32 * 0.05;

            let pushes = (us.pawn >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (us.pawn >> 7) & (opp.pieces | self.en_passant) & !FILE_A;
            let right_attacks = (us.pawn >> 9) & (opp.pieces | self.en_passant) & !FILE_H;

            mobility += pushes.count_ones() as f32  * 0.01 +
                        double_pushes.count_ones() as f32  * -0.01;
            attacks +=  left_attacks.count_ones() as f32   * 0.04 +
                        right_attacks.count_ones() as f32  * 0.04;

            let pushes = (opp.pawn << 8) & !occ;
            let double_pushes = ((pushes & ROW_3) << 8) & !occ;
            let left_attacks = (opp.pawn << 7) & (us.pieces | self.en_passant) & !FILE_H;
            let right_attacks = (opp.pawn << 9) & (us.pieces | self.en_passant) & !FILE_A;

            mobility -= pushes.count_ones() as f32 * 0.01 +
                        double_pushes.count_ones() as f32 * -0.01;
            attacks +=  left_attacks.count_ones() as f32   * 0.04 +
                        right_attacks.count_ones() as f32  * 0.04;
        }

        for_all_pieces(us.queen, &mut |from, piece| {
            let att = queen_attacks(piece, from, occ);
            mobility += (att & !occ).count_ones() as f32 * 0.005;
            attacks += (att & opp.pieces).count_ones() as f32 * 0.015;
        });

        for_all_pieces(us.rook, &mut |from, piece| {
            let att = rook_attacks(piece, from, occ);
            mobility += (att & !occ).count_ones() as f32 * 0.015;
            attacks += (att & opp.pieces).count_ones() as f32 * 0.02;
        });

        for_all_pieces(us.bishop, &mut |from, piece| {
            let att = bishop_attacks(piece, from, occ);
            mobility += (att & !occ).count_ones() as f32 * 0.025;
            attacks += (att & opp.pieces).count_ones() as f32 * 0.03;
        });

        for_all_pieces(us.knight, &mut |from, piece| {
            let att = knight_attacks(from);
            mobility += (att & !occ).count_ones() as f32 * 0.03;
            attacks += (att & opp.pieces).count_ones() as f32 * 0.035;
        });

        for_all_pieces(us.king, &mut |from, piece| {
            let att = king_attacks(from);
            mobility += (att & !occ).count_ones() as f32 * 0.01;
            attacks += (att & opp.pieces).count_ones() as f32 * 0.015;
        });

        // Opponent
        for_all_pieces(opp.queen, &mut |from, piece| {
            let att = queen_attacks(piece, from, occ);
            mobility -= (att & !occ).count_ones() as f32 * 0.005;
            attacks -= (att & opp.pieces).count_ones() as f32 * 0.015;
        });

        for_all_pieces(opp.rook, &mut |from, piece| {
            let att = rook_attacks(piece, from, occ);
            mobility -= (att & !occ).count_ones() as f32 * 0.015;
            attacks -= (att & opp.pieces).count_ones() as f32 * 0.02;
        });

        for_all_pieces(opp.bishop, &mut |from, piece| {
            let att = bishop_attacks(piece, from, occ);
            mobility -= (att & !occ).count_ones() as f32 * 0.025;
            attacks -= (att & opp.pieces).count_ones() as f32 * 0.03;
        });

        for_all_pieces(opp.knight, &mut |from, piece| {
            let att = knight_attacks(from);
            mobility -= (att & !occ).count_ones() as f32 * 0.03;
            attacks -= (att & opp.pieces).count_ones() as f32 * 0.035;
        });

        for_all_pieces(opp.king, &mut |from, piece| {
            let att = king_attacks(from);
            mobility -= (att & !occ).count_ones() as f32 * 0.01;
            attacks -= (att & opp.pieces).count_ones() as f32 * 0.015;
        });

        (us.pawn.count_ones() as f32  * 1.0)   +
        (us.knight.count_ones() as f32 * 3.0)  +
        (us.bishop.count_ones() as f32 * 3.0)  +
        (us.rook.count_ones() as f32 * 5.0)    +
        (us.queen.count_ones() as f32 * 9.0)   +
        (us.king.count_ones() as f32 * 300.0)  -
        (opp.pawn.count_ones() as f32 * 1.0)   -
        (opp.knight.count_ones() as f32 * 3.0) -
        (opp.bishop.count_ones() as f32 * 3.0) -
        (opp.rook.count_ones() as f32 * 5.0)   -
        (opp.queen.count_ones() as f32 * 9.0)  -
        (opp.king.count_ones() as f32 * 300.0) +
        mobility * 0.5 + back_rank + attacks
    }

    pub fn negamax_a_b(&mut self, depth: u32, mut alpha: f32, beta: f32) -> (f32, Move) {
        let mut best = -1000.0;
        let mut best_mv = Move::NULL_MOVE;

        for mv in self.get_pseudo_moves() {
            let mut new_board = self.clone();
            new_board.make_move(mv);
            let (score, submv) = if depth == 1 {
                (self.evaluate(), mv)
            } else {
                let (sub_score, sub_move) = new_board.negamax_a_b(depth - 1, -beta, -alpha);
                (-sub_score, sub_move)
            };

            if score > best {
                best = score;
                best_mv = mv;
            }
            if score > alpha { alpha = score; }
            if score >= beta { return (alpha, best_mv) }
        }
        (best, best_mv)
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

#[derive(Default, Copy, Clone)]
pub struct BitBoard {
    pub pawn: u64,
    pub knight: u64,
    pub bishop: u64,
    pub rook: u64,
    pub queen: u64,
    pub king: u64,
    pub pieces: u64
}
