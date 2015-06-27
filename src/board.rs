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

pub fn add_prom_moves(moves: &mut Vec<Move>, mut targets: u64, diff: i32, flags: u32) {
    while targets != 0 {
        let to = bit_pop_pos(&mut targets);
        let from = ((to as i32) - diff) as u32;

        moves.push(Move::new(from, to, flags | QUEEN_PROM));
        moves.push(Move::new(from, to, flags | ROOK_PROM));
        moves.push(Move::new(from, to, flags | KNIGHT_PROM));
        moves.push(Move::new(from, to, flags | BISHOP_PROM));
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

impl Board {
    pub fn make_move(&mut self, mv: Move) {
        let (src, dest) = (mv.from() as usize, mv.to() as usize);

        let prom = mv.promotion();
        self.sqs[dest] = if prom != 0 {
            let col = if self.move_num % 2 == 1 { Color::White } else { Color::Black };
            match prom {
                QUEEN_PROM  => Square::Piece(PieceType::Queen, col),
                ROOK_PROM   => Square::Piece(PieceType::Rook, col),
                BISHOP_PROM => Square::Piece(PieceType::Bishop, col),
                KNIGHT_PROM => Square::Piece(PieceType::Knight, col),
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
            // If white takes remove from row above, if black takes remove from row below
            let ep_sq = if self.move_num % 1 == 1 { dest + 8 } else { dest - 8 };
            self.sqs[ep_sq] = Square::Empty;
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
        // Make move not copyable
        if is_white {
            let pushes = (us.pawn << 8) & !occ;
            let double_pushes = ((pushes & ROW_3) << 8) & !occ;
            let left_attacks = (us.pawn << 7) & (opp.pieces | self.en_passant) & !FILE_H;
            let right_attacks = (us.pawn << 9) & (opp.pieces | self.en_passant) & !FILE_A;
            let l_en_passant = left_attacks & self.en_passant;
            let r_en_passant = right_attacks & self.en_passant;
            let prom_pushes = pushes & ROW_8;
            let prom_l_att = left_attacks & ROW_8;
            let prom_r_att = right_attacks & ROW_8;

            add_moves(&mut moves, pushes ^ prom_pushes, 8, 0);
            add_moves(&mut moves, double_pushes, 16, DOUBLE_PAWN_PUSH);
            add_moves(&mut moves, left_attacks ^ l_en_passant ^ prom_l_att, 7, IS_CAPTURE);
            add_moves(&mut moves, right_attacks ^ r_en_passant ^ prom_r_att, 9, IS_CAPTURE);
            add_moves(&mut moves, l_en_passant, 7, EN_PASSANT | IS_CAPTURE);
            add_moves(&mut moves, r_en_passant, 9, EN_PASSANT | IS_CAPTURE);
            add_prom_moves(&mut moves, prom_pushes, 8, 0);
            add_prom_moves(&mut moves, prom_l_att, 7, IS_CAPTURE);
            add_prom_moves(&mut moves, prom_r_att, 9, IS_CAPTURE);
        } else {
            let pushes = (us.pawn >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (us.pawn >> 7) & (opp.pieces | self.en_passant) & !FILE_A;
            let right_attacks = (us.pawn >> 9) & (opp.pieces | self.en_passant) & !FILE_H;
            let l_en_passant = left_attacks & self.en_passant;
            let r_en_passant = right_attacks & self.en_passant;
            let prom_pushes = pushes & ROW_1;
            let prom_l_att = left_attacks & ROW_1;
            let prom_r_att = right_attacks & ROW_1;

            add_moves(&mut moves, pushes ^ prom_pushes, -8, 0);
            add_moves(&mut moves, double_pushes, -16, DOUBLE_PAWN_PUSH);
            add_moves(&mut moves, left_attacks ^ l_en_passant ^ prom_l_att, -7, IS_CAPTURE);
            add_moves(&mut moves, right_attacks ^ r_en_passant ^ prom_r_att, -9, IS_CAPTURE);
            add_moves(&mut moves, l_en_passant, -7, EN_PASSANT | IS_CAPTURE);
            add_moves(&mut moves, r_en_passant, -9, EN_PASSANT | IS_CAPTURE);
            add_prom_moves(&mut moves, prom_pushes, -8, 0);
            add_prom_moves(&mut moves, prom_l_att, -7, IS_CAPTURE);
            add_prom_moves(&mut moves, prom_r_att, -9, IS_CAPTURE);
        }

        moves.sort_by(|a,b| {
            if a.is_capture() { Less } else { Greater }
        });

        moves
    }

    pub fn color_to_move(&self) -> Color {
        if self.move_num % 2 == 1 { Color::White } else { Color::Black }
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
            attacks -=  left_attacks.count_ones() as f32   * 0.04 +
                        right_attacks.count_ones() as f32  * 0.04;
        }

        mobility + back_rank + attacks + Board::get_evals(us, opp) - Board::get_evals(opp, us)
    }

    pub fn quiescence_search(&self, depth: u32, mut alpha: f32, beta: f32) -> f32 {
        // TODO: remove depth so all takes are searched
        // TODO: Check for king attacks and break for that branch to avoid illegal moves
        // TODO: When no legal moves possible, return draw to avoid stalemate
        // TODO: Three move repition
        // TODO: Add illegal move detection in queiscence which might cause subtle bugs
        let stand_pat = self.evaluate();
        if depth == 0 { return stand_pat }
        if stand_pat >= beta { return beta }
        if stand_pat > alpha { alpha = stand_pat }

        for mv in self.get_moves().into_iter().filter(|mv| mv.is_capture()) {
            let mut new_board = self.clone();
            new_board.make_move(mv);
            let score = -new_board.quiescence_search(depth - 1, -beta, -alpha);

            if score >= beta { return beta }
            if score > alpha { alpha = score; }
        }
        alpha
    }

    pub fn is_in_check(&self) -> bool {
        // TODO: this isn't super efficient since board is immutable, but only occurs if no legal moves
        let king_pos = self.to_move().king.trailing_zeros();

        let mut clone = self.clone();
        clone.move_num += 1;

        for mv in clone.get_moves() { // get opponent moves
            if mv.to() == king_pos { return true }
        }
        false
    }

    pub fn negamax_a_b(&self, depth: u32, mut alpha: f32, beta: f32, line: &mut Vec<Move>) -> (f32, bool) {
        if depth == 0 { return (self.quiescence_search(4, alpha, beta), true) }
        let mut has_legal_move = false;
        let enemy_king = self.prev_move().king.trailing_zeros();
        let mut localpv = Vec::new();

        for mv in self.get_moves() {
            if mv.to() == enemy_king { return (0.0, false) }
            let mut new_board = self.clone();
            new_board.make_move(mv);

            let (score, is_legal) = new_board.negamax_a_b(depth - 1, -beta, -alpha, &mut localpv);
            let score = -score;

            if is_legal { has_legal_move = true; } else { continue }

            if score >= beta { return (score, true) }
            if score > alpha {
                alpha = score;
                line.clear();
                line.push(mv);
                line.append(&mut localpv);
            }
        }
        if !has_legal_move {
            if self.is_in_check() {
                return (-1000.0 - depth as f32, true)
            } else {
                return (0.0, true)
            }
        }

        (alpha, true)
    }

    pub fn new_fen(fen: &mut Vec<&str>) -> Board {
        let fen_board = fen.remove(0);
        let reversed_rows = fen_board.split('/').rev(); // fen is read from top rank

        let mut sqs = [Square::Empty; 64];

        for (r, row) in reversed_rows.enumerate() {
            let mut offset = 0;
            for (c, ch) in row.chars().enumerate() {
                if !ch.is_numeric() {
                    sqs[r*8 + c+offset] = to_piece(ch);
                } else {
                    offset += (ch as u8 - b'1') as usize;
                }
            }
        }

        let (w, b) = gen_bitboards(&sqs);

        let to_move = fen.remove(0); // Player to move [b,w]
        let move_num = match to_move {
            "w" => 1,
            _ =>   2, // Start of the move counter at an odd number
        };

        let castling = fen.remove(0); // Castling [KQkq]
        let w_k_castle = castling.contains('K');
        let w_q_castle = castling.contains('Q');
        let b_k_castle = castling.contains('k');
        let b_q_castle = castling.contains('q');

        let ep_sq: Vec<char> = fen.remove(0).chars().collect(); // en passant target square
        let en_passant = match ep_sq.as_slice() {
            [sc, sr] => 1 << to_pos(sc, sr),
            _ => 0
        };
        fen.remove(0); // Halfmove Clock
        fen.remove(0); // Fullmove Number

        Board { w: w, b: b, sqs: sqs, move_num: move_num, w_k_castle: w_k_castle, w_q_castle: w_q_castle,
                b_k_castle: b_k_castle, b_q_castle: b_q_castle, en_passant: en_passant }
    }

    pub fn new_default() -> Board {
        let mut def_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".split(' ').collect();
        Board::new_fen(&mut def_fen)
    }
}
