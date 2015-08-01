use std::cmp::Ordering::{Less, Greater};
use types::*;
use util::*;
use magics::*;

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

impl Board {
    pub fn make_move(&mut self, mv: Move) {
        self.hash.set_ep(self.en_passant); // Remove enpessant
        self.hash.flip_color(); // Change color

        let (src, dest) = (mv.from() as usize, mv.to() as usize);
        let color = self.color_to_move();

        self.hash.set_piece(src, self.sqs[src]); // Remove moving piece
        self.hash.set_piece(dest, self.sqs[dest]); // Remove destination piece

        self.sqs[dest] = match mv.promotion() {
            QUEEN_PROM  => QUEEN  | color,
            ROOK_PROM   => ROOK   | color,
            BISHOP_PROM => BISHOP | color,
            KNIGHT_PROM => KNIGHT | color,
            _ => self.sqs[src] // If there is no promotion
        };
        self.sqs[src] = EMPTY;

        self.hash.set_piece(dest, self.sqs[dest]); // Add src piece at dest square

        // TODO: move method
        if mv.king_castle() {
            let color_offset = if color == WHITE { 0 } else { 56 };
            self.hash.set_piece(7 + color_offset, self.sqs[7 + color_offset]);
            self.sqs[7 + color_offset] = EMPTY;
            self.sqs[5 + color_offset] = ROOK | color;
            self.hash.set_piece(5 + color_offset, self.sqs[5 + color_offset]);
        }

        if mv.queen_castle() {
            let color_offset = if color == WHITE { 0 } else { 56 };
            self.hash.set_piece(color_offset, self.sqs[color_offset]);
            self.sqs[color_offset] = EMPTY;
            self.sqs[3 + color_offset] = ROOK | color;
            self.hash.set_piece(3 + color_offset, self.sqs[3 + color_offset]);
        }

        if mv.is_en_passant() {
            // If white takes - remove from row below, if black takes - remove from row above
            let ep_pawn = if color == WHITE { dest - 8 } else { dest + 8 };
            self.sqs[ep_pawn] = EMPTY;
            self.hash.set_piece(ep_pawn, self.sqs[ep_pawn]);
        }

        self.en_passant = 0;
        if mv.is_double_push() {
            self.en_passant = 1 << ((src + dest) / 2);
            self.hash.set_ep(self.en_passant);
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

                let mut flags = match promotion {
                    ['q'] => QUEEN_PROM,
                    ['r'] => ROOK_PROM,
                    ['b'] => BISHOP_PROM,
                    ['n'] => KNIGHT_PROM,
                    _ => 0
                };

                flags |= match mv {
                    "e1g1" if self.castling & Castle::WKing as u8 != 0 => CASTLE_KING,
                    "e8g8" if self.castling & Castle::BKing as u8 != 0 => CASTLE_KING,
                    "e1c1" if self.castling & Castle::WQueen as u8 != 0 => CASTLE_QUEEN,
                    "e8c8" if self.castling & Castle::BQueen as u8 != 0 => CASTLE_QUEEN,
                    _ => 0
                };

                if self.sqs[src as usize] & PIECE == PAWN {
                    let is_double = if src > dest { src-dest } else { dest-src } == 16;
                    if is_double { flags |= DOUBLE_PAWN_PUSH };

                    let is_en_passant = dest == self.en_passant.trailing_zeros();
                    if is_en_passant { flags |= EN_PASSANT };
                }

                self.make_move(Move::new(src, dest, flags));
            },
            _ => panic!("Malformed move {}", mv)
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

        // Move captures to the front to improve move ordering in alpha-beta search
        // Iterative deepening will eventually replace / improve this
        moves.sort_by(|a,b|
            if a.is_capture() { Less } else { Greater }
        );

        moves
    }

    pub fn is_white(&self) -> bool {
        (self.move_num % 2) == 1
    }

    pub fn color_to_move(&self) -> u8 {
        if self.is_white() { WHITE } else { BLACK }
    }

    pub fn to_move(&self) -> &BitBoard {
        if self.is_white() { &self.w } else { &self.b }
    }

    pub fn prev_move(&self) -> &BitBoard {
        if self.is_white() { &self.b } else { &self.w }
    }

    pub fn from_fen(fen: &mut Vec<&str>) -> Board {
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

        let castle_str = fen.remove(0); // Castling [KQkq]
        let mut castling = 0;
        if castle_str.contains('K') { castling |= Castle::WKing as u8  };
        if castle_str.contains('Q') { castling |= Castle::WQueen as u8 };
        if castle_str.contains('k') { castling |= Castle::BKing as u8  };
        if castle_str.contains('q') { castling |= Castle::BQueen as u8 };

        let ep_sq: Vec<char> = fen.remove(0).chars().collect(); // en passant target square
        let en_passant = match ep_sq.as_slice() {
            [sc, sr] => 1 << to_pos(sc, sr),
            _ => 0
        };

        fen.remove(0); // Halfmove Clock
        fen.remove(0); // Fullmove Number

        let mut b = Board { w: w, b: b, sqs: sqs, move_num: move_num, hash: Hash { val: 0 },
                            castling: castling, en_passant: en_passant };

        b.hash = Hash::init(&b);
        b
    }

    pub fn new_default() -> Board {
        let mut def_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".split(' ').collect();
        Board::from_fen(&mut def_fen)
    }
}
