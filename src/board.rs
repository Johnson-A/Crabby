use std::cmp::Ordering::{Less, Greater};
use types::*;
use util::*;
use magics::*;

pub fn gen_bitboards(sqs: &Squares) -> BitBoard {
    let mut bb = BitBoard([0; 14]);

    for (pos, piece) in sqs.iter().enumerate() {
        if *piece != EMPTY { bb[*piece] |= 1 << pos }
    }

    bb[ALL | WHITE] = bb[PAWN | WHITE] | bb[KNIGHT | WHITE] | bb[BISHOP | WHITE] |
                      bb[ROOK | WHITE] | bb[QUEEN | WHITE]  | bb[KING | WHITE];

    // TODO: Don't use | Black since Black = 0
    bb[ALL | BLACK] = bb[PAWN | BLACK] | bb[KNIGHT | BLACK] | bb[BISHOP | BLACK] |
                      bb[ROOK | BLACK] | bb[QUEEN | BLACK]  | bb[KING | BLACK];
    bb
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
    /// Move the specified piece, which may not be the original src piece (when promoting)
    /// Update the board hash correspondingly
    pub fn move_piece(&mut self, src: usize, dest: usize, piece: u8) {
        self.hash.set_piece(src, self.sqs[src]); // Remove moving piece
        self.hash.set_piece(dest, self.sqs[dest]); // Remove destination piece

        self.sqs[src]  = EMPTY;
        self.sqs[dest] = piece;

        self.hash.set_piece(dest, piece); // Add src piece at dest square
    }

    pub fn make_move(&mut self, mv: Move) {
        self.hash.set_ep(self.en_passant); // Remove enpessant
        self.hash.flip_color(); // Change color

        let (src, dest) = (mv.from() as usize, mv.to() as usize);
        let color = self.color_to_move();

        let dest_piece = match mv.promotion() {
            QUEEN_PROM  => QUEEN  | color,
            ROOK_PROM   => ROOK   | color,
            BISHOP_PROM => BISHOP | color,
            KNIGHT_PROM => KNIGHT | color,
            _ => self.sqs[src] // If there is no promotion
        };
        self.move_piece(src, dest, dest_piece);

        if mv.king_castle() {
            let color_offset = if color == WHITE { 0 } else { 56 };
            self.move_piece(7 + color_offset, 5 + color_offset, ROOK | color);
        }

        if mv.queen_castle() {
            let color_offset = if color == WHITE { 0 } else { 56 };
            self.move_piece(color_offset, 3 + color_offset, ROOK | color);
        }

        if mv.is_en_passant() {
            // If white takes -> remove from row below
            // If black takes -> remove from row above
            let ep_pawn = if color == WHITE { dest - 8 } else { dest + 8 };
            self.sqs[ep_pawn] = EMPTY;
            self.hash.set_piece(ep_pawn, self.sqs[ep_pawn]); // Remove taken pawn
        }

        self.en_passant = 0;
        if mv.is_double_push() {
            self.en_passant = 1 << ((src + dest) / 2);
            self.hash.set_ep(self.en_passant);
        }

        // TODO: Update in place
        self.bb = gen_bitboards(&self.sqs);

        self.move_num += 1;
    }

    // TODO:
    pub fn see(&self, mv: &Move) -> isize {
        let src_piece = self.sqs[mv.from() as usize] & PIECE;
        let dest_piece = self.sqs[mv.to() as usize] & PIECE;
        piece_value(dest_piece) - piece_value(src_piece)
    }

    /// Return the lowest value attacker of a given square
    pub fn attacker(&self, pos: u32) -> u8 {
        let bb = &self.bb;
        let dest = 1 << pos;
        let is_white = self.is_white();
        let (us, opp) = (self.to_move(), self.prev_move());

        let l_file = if pos & 8 >= 1 { file(pos - 1) } else { 0 };
        let r_file = if pos & 8 <= 7 { file(pos + 1) } else { 0 };
        let pawns = bb[PAWN | opp] & (l_file | r_file);

        let mut attacks = if !is_white { // Opponent is white
            ((pawns << 7) & dest & !FILE_H) &
            ((pawns << 9) & dest & !FILE_A)
        } else {
            ((bb[PAWN | us] >> 7) & dest & !FILE_A) &
            ((bb[PAWN | us] >> 9) & dest & !FILE_H)
        } != 0;
        if attacks { return PAWN }

        attacks = unsafe { KNIGHT_MAP[pos as usize] } & bb[KNIGHT | opp] != 0;
        if attacks { return KNIGHT }

        let occ = bb[ALL | us] | bb[ALL | opp];

        let row_files = row(pos) & file(pos); // TODO: change col <-> row
        let diagonals = diag(pos) & a_diag(pos);

        let bishops = diagonals & bb[BISHOP | opp];

        for_all_pieces(bishops, &mut |from| {
            let mvs = unsafe { BISHOP_MAP[from as usize].att(occ) };
            attacks |= mvs & dest != 0;
        });
        if attacks { return BISHOP }

        let rooks = row_files & bb[ROOK | opp];

        for_all_pieces(rooks, &mut |from| {
            let mvs = unsafe { ROOK_MAP[from as usize].att(occ) };
            attacks |= mvs & dest != 0;
        });
        if attacks { return ROOK }

        let queens = row_files & diagonals & bb[QUEEN | opp];

        for_all_pieces(queens, &mut |from| {
            let mvs = unsafe {  BISHOP_MAP[from as usize].att(occ) |
                                ROOK_MAP[from as usize].att(occ) };
            attacks |= mvs & dest != 0;
        });
        if attacks { return QUEEN }

        EMPTY
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
        let bb = &self.bb;
        let mut moves: Vec<Move> = Vec::with_capacity(64);

        let is_white = self.is_white();
        let (us, opp) = (self.to_move(), self.prev_move());
        let occ = bb[ALL | us] | bb[ALL | opp];

        for_all_pieces(bb[QUEEN | us], &mut |from| {
            let mvs = unsafe { BISHOP_MAP[from as usize].att(occ) |
                               ROOK_MAP[from as usize].att(occ) };
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & bb[ALL | opp], IS_CAPTURE);
        });

        for_all_pieces(bb[ROOK | us], &mut |from| {
            let mvs = unsafe { ROOK_MAP[from as usize].att(occ) };
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & bb[ALL | opp], IS_CAPTURE);
        });

        for_all_pieces(bb[BISHOP | us], &mut |from| {
            let mvs = unsafe { BISHOP_MAP[from as usize].att(occ) };
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & bb[ALL | opp], IS_CAPTURE);
        });

        for_all_pieces(bb[KNIGHT | us], &mut |from| {
            let mvs = unsafe { KNIGHT_MAP[from as usize] };
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & bb[ALL | opp], IS_CAPTURE);
        });

        for_all_pieces(bb[KING | us], &mut |from| {
            let mvs = unsafe { KING_MAP[from as usize] };
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & bb[ALL | opp], IS_CAPTURE);
        });

        if is_white {
            let pushes = (bb[PAWN | us] << 8) & !occ;
            let double_pushes = ((pushes & ROW_3) << 8) & !occ;
            let left_attacks = (bb[PAWN | us] << 7) & (bb[ALL | opp] | self.en_passant) & !FILE_H;
            let right_attacks = (bb[PAWN | us] << 9) & (bb[ALL | opp] | self.en_passant) & !FILE_A;
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
            let pushes = (bb[PAWN | us] >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (bb[PAWN | us] >> 7) & (bb[ALL | opp] | self.en_passant) & !FILE_A;
            let right_attacks = (bb[PAWN | us] >> 9) & (bb[ALL | opp] | self.en_passant) & !FILE_H;
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
        moves.sort_by(|a,b|
            if self.see(a) > self.see(b) { Less } else { Greater }
        );

        moves
    }

    pub fn is_white(&self) -> bool {
        (self.move_num % 2) == 1
    }

    pub fn color_to_move(&self) -> u8 {
        if self.is_white() { WHITE } else { BLACK }
    }

    pub fn to_move(&self) -> u8 {
        (self.move_num % 2) as u8
    }

    pub fn prev_move(&self) -> u8 {
        ((self.move_num + 1) % 2) as u8
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

        let mut b = Board { bb: gen_bitboards(&sqs), sqs: sqs, move_num: move_num, hash: Hash { val: 0 },
                            castling: castling, en_passant: en_passant };

        b.hash = Hash::init(&b);
        b
    }

    pub fn new_default() -> Board {
        let def_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        Board::from_fen(&mut def_fen.split(' ').collect())
    }
}
