use std::cmp::Ordering::{Less, Greater};
use types::*;
use util::*;

pub fn gen_bitboards(sqs: &Squares) -> (BitBoard, BitBoard) {
    let mut w: BitBoard = Default::default();
    let mut b: BitBoard = Default::default();

    for (pos, sq) in sqs.iter().enumerate() {
        let bb = if sq & COLOR == WHITE { &mut w } else { &mut b };
        match sq & PIECE {
            PAWN   => bb.pawn   |= 1 << pos,
            KNIGHT => bb.knight |= 1 << pos,
            BISHOP => bb.bishop |= 1 << pos,
            ROOK   => bb.rook   |= 1 << pos,
            QUEEN  => bb.queen  |= 1 << pos,
            KING   => bb.king   |= 1 << pos,
            _      => continue
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

pub fn for_all_pieces(mut pieces: u64, do_work: &mut FnMut(u32)) {
    while pieces != 0 {
        let from = bit_pop_pos(&mut pieces);

        do_work(from);
    }
}

impl Board {
    pub fn make_move(&mut self, mv: Move) {
        let (src, dest) = (mv.from() as usize, mv.to() as usize);
        let col = self.color_to_move();

        let prom = mv.promotion();
        self.sqs[dest] = if prom != 0 {
            match prom {
                QUEEN_PROM  => QUEEN  | col,
                ROOK_PROM   => ROOK   | col,
                BISHOP_PROM => BISHOP | col,
                KNIGHT_PROM => KNIGHT | col,
                _ => EMPTY // This can't happen
            }
        } else {
            self.sqs[src]
        };
        self.sqs[src] = EMPTY;

        if mv.king_castle() {
            let color_offset = if col == WHITE { 0 } else { 56 };
            self.sqs[7 + color_offset] = EMPTY;
            self.sqs[5 + color_offset] = ROOK | col;
        }

        if mv.queen_castle() {
            let color_offset = if col == WHITE { 0 } else { 56 };
            self.sqs[color_offset] = EMPTY;
            self.sqs[3 + color_offset] = ROOK | col;
        }

        if mv.is_en_passant() {
            // If white takes remove from row below, if black takes remove from row above
            let ep_pawn = if col == WHITE { dest - 8 } else { dest + 8 };
            self.sqs[ep_pawn] = EMPTY;
        }

        self.en_passant = 0;
        if mv.is_double_push() {
            self.en_passant = 1 << ((src + dest) / 2);
        }

        let (w, b) = gen_bitboards(&self.sqs);
        self.w = w;
        self.b = b;

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

                flags |= match self.sqs[src as usize] & PIECE {
                    PAWN => {
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

        let is_white = self.is_white();
        let (us, opp) = if is_white { (&self.w, &self.b) } else { (&self.b, &self.w) };
        let occ = us.pieces | opp.pieces;

        for_all_pieces(us.queen, &mut |from| {
            let mvs = unsafe { BISHOP_MAP[from as usize].att(occ) |
                               ROOK_MAP[from as usize].att(occ) };
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & opp.pieces, IS_CAPTURE);
        });

        for_all_pieces(us.rook, &mut |from| {
            let mvs = unsafe { ROOK_MAP[from as usize].att(occ) };
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & opp.pieces, IS_CAPTURE);
        });

        for_all_pieces(us.bishop, &mut |from| {
            let mvs = unsafe { BISHOP_MAP[from as usize].att(occ) };
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & opp.pieces, IS_CAPTURE);
        });

        for_all_pieces(us.knight, &mut |from| {
            let mvs = unsafe { KNIGHT_MAP[from as usize] };
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & opp.pieces, IS_CAPTURE);
        });

        for_all_pieces(us.king, &mut |from| {
            let mvs = unsafe { KING_MAP[from as usize] };
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & opp.pieces, IS_CAPTURE);
        });

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

    pub fn is_white(&self) -> bool {
        (self.move_num % 2) == 1
    }

    pub fn color_to_move(&self) -> u8 {
        if self.is_white() { WHITE } else { BLACK }
    }

    pub fn to_move(&self) -> &BitBoard {
        if self.move_num % 2 == 1 { &self.w } else { &self.b }
    }

    pub fn prev_move(&self) -> &BitBoard {
        if self.move_num % 2 == 1 { &self.b } else { &self.w }
    }

    pub fn get_evals(us: &BitBoard, opp: &BitBoard) -> i32 {
        // TODO: remove king material? With legal move checking, and mate and stalemate now added
        let occ = us.pieces | opp.pieces;

        let mut eval = 0;

        for_all_pieces(us.queen, &mut |from| {
            let att = unsafe { BISHOP_MAP[from as usize].att(occ) |
                               ROOK_MAP[from as usize].att(occ) };
            eval += (att & !occ).count_ones() * 5 +
                    (att & opp.pieces).count_ones() * 15 +
                    (att & us.pieces).count_ones() * 8;
        });

        for_all_pieces(us.rook, &mut |from| {
            let att = unsafe { ROOK_MAP[from as usize].att(occ) };
            eval += (att & !occ).count_ones() * 15 +
                    (att & opp.pieces).count_ones() * 20 +
                    (att & us.pieces).count_ones() * 10;
        });

        for_all_pieces(us.bishop, &mut |from| {
            let att = unsafe { BISHOP_MAP[from as usize].att(occ) };
            eval += (att & !occ).count_ones() * 25 +
                    (att & opp.pieces).count_ones() * 30 +
                    (att & us.pieces).count_ones() * 10;
        });

        for_all_pieces(us.knight, &mut |from| {
            let att = unsafe { KNIGHT_MAP[from as usize] };
            eval += (att & !occ).count_ones() * 30 +
                    (att & opp.pieces).count_ones() * 35 +
                    (att & us.pieces).count_ones() * 12;
        });

        for_all_pieces(us.king, &mut |from| {
            let att = unsafe { KING_MAP[from as usize] };
            eval += (att & !occ).count_ones() * 10 +
                    (att & opp.pieces).count_ones() * 15 +
                    (att & us.pieces).count_ones() * 10;
        });

        let material =  (us.pawn.count_ones()   * 1000)  +
                        (us.knight.count_ones() * 3000)  +
                        (us.bishop.count_ones() * 3000)  +
                        (us.rook.count_ones()   * 5000)  +
                        (us.queen.count_ones()  * 9000)  +
                        (us.king.count_ones()   * 300000);

        (material + eval) as i32
    }

    pub fn evaluate(&self) -> i32 {
        // TODO: Don't trade if material down or in worse position
        // TODO: doubled pawns
        // TODO: Center squares and pawns
        let opp = self.prev_move();
        let us = self.to_move(); // Node player

        let mut eval = 1000*1000;

        let is_white = self.is_white();
        let occ = us.pieces | opp.pieces;

        if is_white {
            eval -= ((us.pieces ^ (us.king | us.queen)) & ROW_1).count_ones() * 50;
            eval += ((opp.pieces ^ (opp.king | opp.queen)) & ROW_8).count_ones() * 50;

            let pushes = (us.pawn << 8) & !occ;
            let double_pushes = ((pushes & ROW_3) << 8) & !occ;
            let left_attacks = (us.pawn << 7) & (opp.pieces | self.en_passant) & !FILE_H;
            let right_attacks = (us.pawn << 9) & (opp.pieces | self.en_passant) & !FILE_A;

            eval += pushes.count_ones() * 10 +
                    double_pushes.count_ones() * 10;
            eval += left_attacks.count_ones() * 40 +
                    right_attacks.count_ones() * 40;

            let pushes = (opp.pawn >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (opp.pawn >> 7) & (us.pieces | self.en_passant) & !FILE_A;
            let right_attacks = (opp.pawn >> 9) & (us.pieces | self.en_passant) & !FILE_H;

            eval -= pushes.count_ones() * 10 +
                    double_pushes.count_ones() * 10;
            eval -= left_attacks.count_ones() * 40 +
                    right_attacks.count_ones() * 40;
        } else {
            eval -= ((us.pieces ^ (us.king | us.queen)) & ROW_8).count_ones() * 50;
            eval += ((opp.pieces ^ (opp.king | opp.queen)) & ROW_1).count_ones() * 50;

            let pushes = (us.pawn >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (us.pawn >> 7) & (opp.pieces | self.en_passant) & !FILE_A;
            let right_attacks = (us.pawn >> 9) & (opp.pieces | self.en_passant) & !FILE_H;

            eval += pushes.count_ones() * 10 +
                    double_pushes.count_ones() * 10;
            eval += left_attacks.count_ones() * 40 +
                    right_attacks.count_ones() * 40;

            let pushes = (opp.pawn << 8) & !occ;
            let double_pushes = ((pushes & ROW_3) << 8) & !occ;
            let left_attacks = (opp.pawn << 7) & (us.pieces | self.en_passant) & !FILE_H;
            let right_attacks = (opp.pawn << 9) & (us.pieces | self.en_passant) & !FILE_A;

            eval -= pushes.count_ones() * 10 +
                    double_pushes.count_ones() * 10;
            eval -= left_attacks.count_ones() * 40 +
                    right_attacks.count_ones() * 40;
        }

        let real_eval = (eval as i32) - 1000*1000;
        real_eval + Board::get_evals(us, opp) - Board::get_evals(opp, us)
    }

    pub fn quiescence_search(&self, depth: u32, mut alpha: i32, beta: i32) -> i32 {
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

    pub fn negamax_a_b(&self, depth: u32, mut alpha: i32, beta: i32, line: &mut Vec<Move>) -> (i32, bool) {
        if depth == 0 { return (self.quiescence_search(4, alpha, beta), true) }
        let mut has_legal_move = false;
        let enemy_king = self.prev_move().king.trailing_zeros();
        let mut localpv = Vec::new();

        for mv in self.get_moves() {
            if mv.to() == enemy_king { return (0, false) }
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
                return (-1000000 - depth as i32, true)
            } else {
                return (0, true)
            }
        }

        (alpha, true)
    }

    pub fn new_fen(fen: &mut Vec<&str>) -> Board {
        let fen_board = fen.remove(0);
        let reversed_rows = fen_board.split('/').rev(); // fen is read from top rank

        let mut sqs = [EMPTY; 64];

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
