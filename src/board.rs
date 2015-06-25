use std::fmt;
use std::cmp::Ordering::{Less, Equal, Greater};
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

pub fn add_moves(moves: &mut Vec<Move>, mut targets: u64, diff: i32, flags: u32) {
    while targets != 0 {
        let to = bit_pop_pos(&mut targets);
        let from = ((to as i32) - diff) as u32;
        // let capture = board
        moves.push(Move::new(from, to, flags));
    }
}

pub fn add_moves_from(moves: &mut Vec<Move>, from: u32, mut targets: u64, flags: u32) {
    while targets != 0 {
        let to = bit_pop_pos(&mut targets);
        moves.push(Move::new(from, to, flags));
    }
}

pub fn for_all_pieces(mut pieces: u64, do_work: &mut FnMut(u32, u64)) {
    while pieces != 0 {
        let piece = bit_pop(&mut pieces);
        let from = piece.trailing_zeros();

        do_work(from, piece);
    }
}

#[derive(Copy)]
pub struct Board {
    pub w: BitBoard,
    pub b: BitBoard,
    pub sqs: Squares,
    pub move_num: u32,
    pub w_k_castle: bool,
    pub w_q_castle: bool,
    pub b_k_castle: bool,
    pub b_q_castle: bool,
    pub en_passant: u64
}

impl Clone for Board { fn clone(&self) -> Self { *self } }

impl Board {
    pub fn make_move(&mut self, mv: Move) {
        let (src, dest) = (mv.from() as usize, mv.to() as usize);

        let prom = mv.promotion();
        self.sqs[dest] = if prom != 0 {
            let col = if self.move_num % 2 == 1 { Color::White } else { Color::Black };
            match prom {
                QUEEN_PROM  => Square::Piece(PieceType::Queen, col),
                ROOK_PROM   => Square::Piece(PieceType::Queen, col),
                BISHOP_PROM => Square::Piece(PieceType::Queen, col),
                KNIGHT_PROM => Square::Piece(PieceType::Queen, col),
                _ => Square::Empty // This can't happen
            }
        } else {
            self.sqs[src]
        };
        self.sqs[src] = Square::Empty;

        if mv.king_castle() {
            let color_offset = if self.move_num % 2 == 1 { 0 } else { 56 };
            self.sqs[7 + color_offset] = Square::Empty;
            self.sqs[5 + color_offset] = Square::Piece(PieceType::Rook, self.color_to_move());
        }

        if mv.queen_castle() {
            let color_offset = if self.move_num % 2 == 1 { 0 } else { 56 };
            self.sqs[color_offset] = Square::Empty;
            self.sqs[3 + color_offset] = Square::Piece(PieceType::Rook, self.color_to_move());
        }

        if mv.is_en_passant() {
            let diff = if self.move_num % 1 == 0 { -8 } else { 8 };
            println!("{}", dest + diff);
            self.sqs[dest + diff] = Square::Empty;
        }

        self.en_passant = 0;
        if mv.is_double_push() {
            self.en_passant = 1 << ((src + dest) / 2);
        }

        let (w, b) = gen_bitboards(&self.sqs);
        self.w = w;
        self.b = b;
        // Board::update(&mut self.w, &mut self.b, mv);
        self.move_num += 1;
    }

    pub fn color_to_move(&self) -> Color {
        if self.move_num % 2 == 1 { Color::White } else { Color::Black }
    }

    pub fn make_str_move(&mut self, mv: &str) {
        let moves: Vec<char> = mv.chars().collect();
        match moves.as_slice() {
            [sc, sr, dc, dr, promotion..] => {
                let (src, dest) = (to_pos(sc, sr), to_pos(dc, dr));

                let mut flags = if promotion.len() == 1 {
                    match promotion[0] {
                        'q' => QUEEN_PROM,
                        'r' => ROOK_PROM,
                        'b' => BISHOP_PROM,
                        'n' => KNIGHT_PROM,
                        _ => 0
                    }
                } else { 0 };

                flags |= match mv {
                    "e1g1" => if self.w_k_castle { CASTLE_KING } else { 0 },
                    "e8g8" => if self.b_k_castle { CASTLE_KING } else { 0 },
                    "e1c1" => if self.w_q_castle { CASTLE_QUEEN } else { 0 },
                    "e8c8" => if self.b_q_castle { CASTLE_QUEEN } else { 0 },
                    _ => 0
                };

                flags |= match self.sqs[src as usize] {
                    Square::Piece(PieceType::Pawn, _) => {
                        let is_double = if src > dest { src-dest } else { dest-src } == 16;
                        let is_en_passant = dest == self.en_passant.trailing_zeros();

                        (if is_en_passant { EN_PASSANT } else { 0 }) |
                        (if is_double { DOUBLE_PAWN_PUSH } else { 0 })
                    },
                    _ => 0
                };

                self.make_move(Move::new(src, dest, flags));
            },
            _ => () // malformed move
        }
    }

    pub fn get_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::with_capacity(64);

        let is_white = self.move_num % 2 == 1;
        let (us, opp) = if is_white { (&self.w, &self.b) } else { (&self.b, &self.w) };

        let occ = us.pieces | opp.pieces;

        for_all_pieces(us.queen, &mut |from, piece| {
            let mvs = queen_attacks(piece, from, occ);
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & opp.pieces, IS_CAPTURE);
        });

        for_all_pieces(us.rook, &mut |from, piece| {
            let mvs = rook_attacks(piece, from, occ);
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & opp.pieces, IS_CAPTURE);
        });

        for_all_pieces(us.bishop, &mut |from, piece| {
            let mvs = bishop_attacks(piece, from, occ);
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & opp.pieces, IS_CAPTURE);
        });

        for_all_pieces(us.knight, &mut |from, piece| {
            let mvs = knight_attacks(from);
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & opp.pieces, IS_CAPTURE);
        });

        for_all_pieces(us.king, &mut |from, piece| {
            let mvs = king_attacks(from);
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & opp.pieces, IS_CAPTURE);
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
            let l_en_passant = left_attacks & self.en_passant;
            let r_en_passant = right_attacks & self.en_passant;
            let promotions = pushes & ROW_8;
            pushes &= !ROW_8;

            add_moves(&mut moves, pushes, 8, 0);
            add_moves(&mut moves, double_pushes, 16, DOUBLE_PAWN_PUSH);
            add_moves(&mut moves, left_attacks ^ l_en_passant, 7, IS_CAPTURE);
            add_moves(&mut moves, right_attacks ^ r_en_passant, 9, IS_CAPTURE);
            add_moves(&mut moves, l_en_passant, 7, EN_PASSANT | IS_CAPTURE);
            add_moves(&mut moves, r_en_passant, 9, EN_PASSANT | IS_CAPTURE);
            add_moves(&mut moves, promotions, 8, 0);
        } else {
            let mut pushes = (us.pawn >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (us.pawn >> 7) & (opp.pieces | self.en_passant) & !FILE_A;
            let right_attacks = (us.pawn >> 9) & (opp.pieces | self.en_passant) & !FILE_H;
            let l_en_passant = left_attacks & self.en_passant;
            let r_en_passant = right_attacks & self.en_passant;
            let promotions = pushes & ROW_1;
            pushes &= !ROW_1;

            add_moves(&mut moves, pushes, -8, 0);
            add_moves(&mut moves, double_pushes, -16, DOUBLE_PAWN_PUSH);
            add_moves(&mut moves, left_attacks ^ l_en_passant, -7, IS_CAPTURE);
            add_moves(&mut moves, right_attacks ^ r_en_passant, -9, IS_CAPTURE);
            add_moves(&mut moves, l_en_passant, -7, EN_PASSANT | IS_CAPTURE);
            add_moves(&mut moves, r_en_passant, -9, EN_PASSANT | IS_CAPTURE);
            add_moves(&mut moves, promotions, -8, 0);
        }

        moves.sort_by(|a,b| {
            if a.is_capture() { Less } else { Greater }
            });
        // println!("moves = {:?}", moves.iter().map(|mv| mv.flags()).collect::<Vec<u32>>());
        // println!("{:?}", moves.iter().map(|a| a.is_capture()).collect::<Vec<bool>>());
        moves
    }

    pub fn to_move(&self) -> &BitBoard {
        if self.move_num % 2 == 1 { &self.w } else { &self.b }
    }

    pub fn prev_move(&self) -> &BitBoard {
        if self.move_num % 2 == 1 { &self.b } else { &self.w }
    }

    pub fn get_evals(us: &BitBoard, opp: &BitBoard) -> f32 {
        let occ = us.pieces | opp.pieces;

        let mut mobility = 0.0;
        let mut attacks = 0.0;
        let mut defenses = 0.0;

        for_all_pieces(us.queen, &mut |from, piece| {
            let att = queen_attacks(piece, from, occ);
            mobility += (att & !occ).count_ones() as f32 * 0.005;
            attacks += (att & opp.pieces).count_ones() as f32 * 0.015;
            defenses += (att & us.pieces).count_ones() as f32 * 0.0075;
        });

        for_all_pieces(us.rook, &mut |from, piece| {
            let att = rook_attacks(piece, from, occ);
            mobility += (att & !occ).count_ones() as f32 * 0.015;
            attacks += (att & opp.pieces).count_ones() as f32 * 0.02;
            defenses += (att & us.pieces).count_ones() as f32 * 0.01;
        });

        for_all_pieces(us.bishop, &mut |from, piece| {
            let att = bishop_attacks(piece, from, occ);
            mobility += (att & !occ).count_ones() as f32 * 0.025;
            attacks += (att & opp.pieces).count_ones() as f32 * 0.03;
            defenses += (att & us.pieces).count_ones() as f32 * 0.01;
        });

        for_all_pieces(us.knight, &mut |from, piece| {
            let att = knight_attacks(from);
            mobility += (att & !occ).count_ones() as f32 * 0.03;
            attacks += (att & opp.pieces).count_ones() as f32 * 0.035;
            defenses += (att & us.pieces).count_ones() as f32 * 0.0125;
        });

        for_all_pieces(us.king, &mut |from, piece| {
            let att = king_attacks(from);
            mobility += (att & !occ).count_ones() as f32 * 0.01;
            attacks += (att & opp.pieces).count_ones() as f32 * 0.015;
            defenses += (att & us.pieces).count_ones() as f32 * 0.01;
        });

        let material =  (us.pawn.count_ones() as f32  * 1.0)   +
                        (us.knight.count_ones() as f32 * 3.0)  +
                        (us.bishop.count_ones() as f32 * 3.0)  +
                        (us.rook.count_ones() as f32 * 5.0)    +
                        (us.queen.count_ones() as f32 * 9.0)   +
                        (us.king.count_ones() as f32 * 300.0);

        material + mobility + attacks + defenses
    }

    pub fn evaluate(&self) -> f32 {
        // TODO: Don't trade if material down or in worse position
        // TODO: doubled pawns
        // TODO: Center squares and pawns
        let opp = self.prev_move();
        let us = self.to_move(); // Node player

        let mut mobility = 0.0;
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
                        double_pushes.count_ones() as f32 * 0.01;
            attacks +=  left_attacks.count_ones() as f32   * 0.04 +
                        right_attacks.count_ones() as f32  * 0.04;

            let pushes = (opp.pawn >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (opp.pawn >> 7) & (us.pieces | self.en_passant) & !FILE_A;
            let right_attacks = (opp.pawn >> 9) & (us.pieces | self.en_passant) & !FILE_H;

            mobility -= pushes.count_ones() as f32  * 0.01 +
                        double_pushes.count_ones() as f32  * 0.01;
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
                        double_pushes.count_ones() as f32  * 0.01;
            attacks +=  left_attacks.count_ones() as f32   * 0.04 +
                        right_attacks.count_ones() as f32  * 0.04;

            let pushes = (opp.pawn << 8) & !occ;
            let double_pushes = ((pushes & ROW_3) << 8) & !occ;
            let left_attacks = (opp.pawn << 7) & (us.pieces | self.en_passant) & !FILE_H;
            let right_attacks = (opp.pawn << 9) & (us.pieces | self.en_passant) & !FILE_A;

            mobility -= pushes.count_ones() as f32 * 0.01 +
                        double_pushes.count_ones() as f32 * 0.01;
            attacks +=  left_attacks.count_ones() as f32   * 0.04 +
                        right_attacks.count_ones() as f32  * 0.04;
        }

        mobility + back_rank + attacks + Board::get_evals(us, opp) - Board::get_evals(opp, us)
    }

    pub fn quiescence_search(&self, depth: u32, mut alpha: f32, beta: f32) -> f32 {
        // TODO: remove depth so all takes are searched
        // TODO: Check for king attacks and break
        let stand_pat = self.evaluate();
        if depth == 0 { return stand_pat }
        // println!("{} {} {}", stand_pat, alpha, beta);
        if stand_pat >= beta { return beta }
        if alpha < stand_pat { alpha = stand_pat }

        // filter on is captured
        for mv in self.get_moves().into_iter().filter(|mv| mv.is_capture()) {
            // println!("{}\n{} {}", self, mv.from(), mv.to());
            let mut new_board = self.clone();
            new_board.make_move(mv);
            let score = -new_board.quiescence_search(depth - 1, -beta, -alpha);

            if score > alpha { alpha = score; }
            if score >= beta { return beta }
        }
        alpha
    }

    pub fn negamax_a_b(&self, depth: u32, mut alpha: f32, beta: f32) -> (f32, Move) {
        let mut best = -1000.0;
        let mut best_mv = Move::NULL_MOVE;

        for mv in self.get_moves() {
            let mut new_board = self.clone();
            new_board.make_move(mv);

            let (score, submv) = if depth == 1 {
                // (-new_board.evaluate(), mv)
                // (-new_board.quiescence_search(4, -10000.0, 10000.0), mv)
                (-new_board.quiescence_search(4, -beta, -alpha), mv)
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

        Board { w: w, b: b, sqs: sqs, move_num: 1, w_k_castle: true, w_q_castle: true,
                b_k_castle: true, b_q_castle: true, en_passant: 0 }
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
        write!(f, "--------\n{}--------\n\
                  Move # {:?}\n\
                  wkcas {} wqcas {} bkcas {} bqcas {}\nen passant {}",
                  output, self.move_num,
                  self.w_k_castle, self.w_q_castle, self.b_k_castle, self.b_q_castle, self.en_passant)
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
