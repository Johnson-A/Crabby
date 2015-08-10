use std::cmp::Ordering::{Less, Greater};
use std::cmp::max;
use types::*;
use util::*;
use magics::*;

pub fn gen_bitboards(sqs: &Squares) -> BitBoard {
    let mut bb = BitBoard([0; 14]);

    for (pos, piece) in sqs.iter().enumerate() {
        if *piece != EMPTY { bb[*piece] |= 1 << pos }
    }

    bb.set_all();

    bb
}

pub fn add_moves(moves: &mut Vec<Move>, mut targets: u64, diff: i32, flags: u32) {
    while targets != 0 {
        let to = bit_pop(&mut targets);
        let from = (to as i32 - diff) as u32;
        moves.push(Move::new(from, to, flags));
    }
}

pub fn add_moves_from(moves: &mut Vec<Move>, from: u32, mut targets: u64, flags: u32) {
    while targets != 0 {
        let to = bit_pop(&mut targets);
        moves.push(Move::new(from, to, flags));
    }
}

pub fn add_prom_moves(moves: &mut Vec<Move>, mut targets: u64, diff: i32, flags: u32) {
    while targets != 0 {
        let to = bit_pop(&mut targets);
        let from = (to as i32 - diff) as u32;

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
        let (src_pc, dest_pc) = (self.sqs[src], self.sqs[dest]);
        self.hash.set_piece(src, src_pc); // Remove moving piece
        self.hash.set_piece(dest, dest_pc); // Remove destination piece
        self.bb[src_pc] ^= 1 << src;
        if dest_pc != EMPTY { self.bb[dest_pc] ^= 1 << dest }

        self.sqs[src]  = EMPTY;
        self.sqs[dest] = piece;

        self.hash.set_piece(dest, piece); // Add src piece at dest square
        self.bb[piece] ^= 1 << dest;
    }

    pub fn make_move(&mut self, mv: Move) {
        self.hash.set_ep(self.en_passant); // Remove enpessant
        self.hash.flip_color(); // Change color

        let (src, dest) = (mv.from() as usize, mv.to() as usize);
        let color = self.to_move();

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
            // If white takes -> remove from row below. If black takes -> remove from row above
            let ep_pawn = if color == WHITE { dest - 8 } else { dest + 8 };
            self.hash.set_piece(ep_pawn, self.sqs[ep_pawn]); // Remove taken pawn
            self.bb[PAWN | flip(color)] ^= 1 << ep_pawn;
            self.sqs[ep_pawn] = EMPTY;
        }

        self.en_passant = 0;
        if mv.is_double_push() {
            self.en_passant = 1 << ((src + dest) / 2);
            self.hash.set_ep(self.en_passant);
        }

        self.bb.set_all();

        self.move_num += 1;
    }

    pub fn see(&mut self, pos: u32, us: u8) -> i32 {
        let (piece, from) = self.attacker(pos, us);

        if piece != EMPTY {
            self.move_piece(from as usize, pos as usize, piece);
            max(0, p_val(piece) as i32 - self.see(pos, flip(us)))
        } else {
            0
        }
    }

    pub fn see_move(&mut self, mv: &Move) -> i32 {
        if !mv.is_capture() { return 0 }
        let us = self.to_move();
        let capture = self.sqs[mv.to() as usize];

        self.make_move(*mv);
        p_val(capture) as i32 - self.see(mv.to(), us)
    }

    pub fn see_max_one(&mut self, mv: &Move) -> i32 {
        if !mv.is_capture() { return 0 }
        let us = self.to_move();

        let src_piece = self.sqs[mv.from() as usize];
        let dest_piece = self.sqs[mv.to() as usize];
        let (defender, _) = self.attacker(mv.to(), us);

        if defender == EMPTY {
            p_val(dest_piece) as i32
        } else {
            p_val(dest_piece) as i32 - p_val(src_piece) as i32
        }
    }

    /// Return the lowest valued enemy attacker of a given square and its position
    pub fn attacker(&self, pos: u32, us: u8) -> (u8, u32) {
        let bb = &self.bb;
        let dest = 1 << pos;
        let opp = flip(us);

        let l_file = if pos & 8 >= 1 { file(pos - 1) } else { 0 };
        let r_file = if pos & 8 <= 7 { file(pos + 1) } else { 0 };
        let pawns = bb[PAWN | opp] & (l_file | r_file);

        // TODO: improve logic like in get_moves
        if us == BLACK { // Opponent is white
            if (pawns << 7) & dest & !FILE_H != 0 { return (PAWN | us, pos - 7) }
            if (pawns << 9) & dest & !FILE_A != 0 { return (PAWN | us, pos - 9) }
        } else {
            if (pawns >> 7) & dest & !FILE_A != 0 { return (PAWN | us, pos + 7) }
            if (pawns >> 9) & dest & !FILE_H != 0 { return (PAWN | us, pos + 9) }
        };

        let knight = unsafe { KNIGHT_MAP[pos as usize] } & bb[KNIGHT | opp];
        if knight != 0 { return (KNIGHT | us, lsb(knight)) }

        let occ = bb[ALL | us] | bb[ALL | opp];

        let row_files = row(pos) | file(pos);
        let diagonals = diag(pos) | a_diag(pos);

        let mut bishops = diagonals & bb[BISHOP | opp];

        while bishops != 0 {
            let from = bit_pop(&mut bishops);
            let mvs = unsafe { BISHOP_MAP[from as usize].att(occ) };
            if mvs & dest != 0 { return (BISHOP | us, from) }
        }

        let mut rooks = row_files & bb[ROOK | opp];

        while rooks != 0 {
            let from = bit_pop(&mut rooks);
            let mvs = unsafe { ROOK_MAP[from as usize].att(occ) };
            if mvs & dest != 0 { return (ROOK | us, from) }
        }

        let mut queens = (row_files | diagonals) & bb[QUEEN | opp];

        while queens != 0 {
            let from = bit_pop(&mut queens);
            let mvs = unsafe { BISHOP_MAP[from as usize].att(occ) |
                               ROOK_MAP[from as usize].att(occ) };
            if mvs & dest != 0 { return (QUEEN | us, from) }
        }

        // TODO: once fixed
        // let from = bb[KING | us].trailing_zeros();
        // let mvs = unsafe { KING_MAP[from as usize] };
        // if mvs & dest != 0 { return KING }

        (EMPTY, !0)
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

        let (us, opp) = (self.to_move(), self.prev_move());
        let enemies = bb[ALL | opp];
        let occ = bb[ALL | us] | enemies;

        let (rank_3, rank_8, l_file, r_file, up, left, right) =
            if us == WHITE { PAWN_WHITE } else { PAWN_BLACK };

        for_all(bb[QUEEN | us], &mut |from| {
            let mvs = unsafe { BISHOP_MAP[from as usize].att(occ) |
                               ROOK_MAP[from as usize].att(occ) };
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & enemies, IS_CAPTURE);
        });

        for_all(bb[ROOK | us], &mut |from| {
            let mvs = unsafe { ROOK_MAP[from as usize].att(occ) };
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & enemies, IS_CAPTURE);
        });

        for_all(bb[BISHOP | us], &mut |from| {
            let mvs = unsafe { BISHOP_MAP[from as usize].att(occ) };
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & enemies, IS_CAPTURE);
        });

        for_all(bb[KNIGHT | us], &mut |from| {
            let mvs = unsafe { KNIGHT_MAP[from as usize] };
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & enemies, IS_CAPTURE);
        });

        // since the king can be taken in the quiescence search, the king bit board can be 0
        // for a copy of board which is being used for queiscence search. for_all uses
        // while piece != 0 so we never access KING_MAP with an invalid key. without the while != 0,
        // the lsb is "64" causing an error
        for_all(bb[KING | us], &mut |from| {
            let mvs = unsafe { KING_MAP[from as usize] };
            add_moves_from(&mut moves, from, mvs & !occ, 0);
            add_moves_from(&mut moves, from, mvs & enemies, IS_CAPTURE);
        });
        // let from = lsb(bb[KING | us]);
        // let test = unsafe { KING_MAP[from as usize] };
        // add_moves_from(&mut moves, from, mvs & !occ, 0);
        // add_moves_from(&mut moves, from, mvs & enemies, IS_CAPTURE);

        let (pushes, double_pushes, left_attacks, right_attacks);
        let pawns = bb[PAWN | us];

        if us == WHITE {
            pushes = (pawns << up) & !occ;
            double_pushes = ((pushes & rank_3) << up) & !occ;
            left_attacks = (pawns << left) & (enemies | self.en_passant) & !r_file;
            right_attacks = (pawns << right) & (enemies | self.en_passant) & !l_file;
        } else {
            pushes = (pawns >> -up) & !occ;
            double_pushes = ((pushes & rank_3) >> -up) & !occ;
            left_attacks = (pawns >> -left) & (enemies | self.en_passant) & !r_file;
            right_attacks = (pawns >> -right) & (enemies | self.en_passant) & !l_file;
        }
        let l_en_passant = left_attacks & self.en_passant;
        let r_en_passant = right_attacks & self.en_passant;
        let prom_pushes = pushes & rank_8;
        let prom_l_att = left_attacks & rank_8;
        let prom_r_att = right_attacks & rank_8;

        add_moves(&mut moves, pushes ^ prom_pushes, up, 0);
        add_moves(&mut moves, double_pushes, up+up, DOUBLE_PAWN_PUSH);
        add_moves(&mut moves, left_attacks ^ l_en_passant ^ prom_l_att, left, IS_CAPTURE);
        add_moves(&mut moves, right_attacks ^ r_en_passant ^ prom_r_att, right, IS_CAPTURE);
        add_moves(&mut moves, l_en_passant, left, EN_PASSANT | IS_CAPTURE);
        add_moves(&mut moves, r_en_passant, right, EN_PASSANT | IS_CAPTURE);
        add_prom_moves(&mut moves, prom_pushes, up, 0);
        add_prom_moves(&mut moves, prom_l_att, left, IS_CAPTURE);
        add_prom_moves(&mut moves, prom_r_att, right, IS_CAPTURE);

        moves
    }

    /// Move better SEE to the front to improve move ordering in alpha-beta search
    pub fn sort(&self, moves: &Vec<Move>) -> Vec<(i32, Move)> {
        let mut temp: Vec<(i32, Move)> = moves.iter().map(
            |mv| (self.clone().see_move(mv), *mv)).collect();
        temp.sort_by(|a,b|
            if a.0 > b.0 { Less } else { Greater }
        );

        temp
    }

    pub fn is_white(&self) -> bool {
        (self.move_num % 2) == 1
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
