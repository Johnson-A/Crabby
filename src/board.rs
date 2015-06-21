use std::fmt;
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

pub struct Move { data: u32 }

impl Move {
    pub fn new(from: u32, to: u32, flags: u32) -> Move {
        let d = from | to << 6 | flags << 12;
        Move { data: d }
    }

    pub fn from(&self)  -> u32 { self.data & 0x3F }
    pub fn to(&self)    -> u32 { (self.data >> 6) & 0x3F }
    pub fn flags(&self) -> u32 { (self.data >> 12) & 0x3F }
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

pub fn add_moves(moves: &mut Vec<Move>, mut targets: u64, diff: u32) {
    while targets != 0 {
        let to = bit_pop_pos(&mut targets);
        let from = to - diff;
        // let capture = board
        moves.push(Move::new(from, to, 0));
    }
}

pub fn add_moves_from(moves: &mut Vec<Move>, mut targets: u64, from: u32) {
    while targets != 0 {
        let to = bit_pop_pos(&mut targets);
        moves.push(Move::new(from, to, 0));
    }
}

pub fn get_line_attacks(occ: u64, mask: u64, piece: u64) -> u64 {
    let pot_blockers = occ & mask;
    let forward = pot_blockers - 2*piece;
    let rev = reverse(reverse(pot_blockers) - 2*reverse(piece));
    (forward ^ rev) & mask
}

pub struct Board {
    pub w: BitBoard,
    pub b: BitBoard,
    pub sqs: Squares,
}

impl Board {
    pub fn make_move(&mut self, src: usize, dest: usize) {
        self.sqs[dest] = self.sqs[src];
        self.sqs[src] = Square::Empty;
        let (w, b) = gen_bitboards(&self.sqs);
        self.w = w;
        self.b = b;
    }

    pub fn make_str_move(&mut self, mv: &str) {
        let moves: Vec<char> = mv.chars().collect();
        match moves.as_slice() {
            [sc, sr, dc, dr] => {
                let src_pos = to_pos(sc, sr);
                let dest_pos = to_pos(dc, dr);
                self.make_move(src_pos, dest_pos);
            },
            _ => () // malformed move
        }
    }

    pub fn get_moves(&self, color: Color) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::with_capacity(64);

        let white_side = color == Color::White;
        let (us, opp) = if white_side { (&self.w, &self.b) } else { (&self.b, &self.w) };
        let rank_3    = if white_side { ROW_3 } else { ROW_6 };
        let prom_rank = if white_side { ROW_8 } else { ROW_1 };

        let occ = us.pieces | opp.pieces;
        
        // Pawn
        let mut pushes = (us.pawn << 8) & !occ;

        let double_pushes = ((pushes & rank_3) << 8) & !occ;

        let left_attacks = (us.pawn << 7) & opp.pieces & !FILE_H;

        let right_attacks = (us.pawn << 9) & opp.pieces & !FILE_A;

        let promotions = pushes & prom_rank; // Get all moves
        // let promotion_captures = (left_attacks | right_attacks) & ROW_8;
        pushes &= !prom_rank; // Remove ROW_8 pushes

        // En Pessant goes here

        add_moves(&mut moves, pushes, 8);
        add_moves(&mut moves, double_pushes, 16);
        add_moves(&mut moves, left_attacks, 7);
        add_moves(&mut moves, right_attacks, 9);
        add_moves(&mut moves, promotions, 8);

        let mut queen_bb = us.queen;
        while queen_bb != 0 {
            let piece = bit_pop(&mut queen_bb);
            let from = piece.trailing_zeros();

            let attacks = get_line_attacks(occ, file(from), piece) |
                          get_line_attacks(occ, row(from),  piece) |
                          get_line_attacks(occ, diag(from), piece) |
                          get_line_attacks(occ, a_diag(from), piece);

            let qmoves = attacks & !us.pieces;
            add_moves_from(&mut moves, qmoves, from);
        }

        let mut rook_bb = us.rook;
        while rook_bb != 0 {
            let piece = bit_pop(&mut rook_bb);
            let from = piece.trailing_zeros();

            let attacks = get_line_attacks(occ, file(from), piece) |
                          get_line_attacks(occ, row(from), piece);

            let rmoves = attacks & !us.pieces;
            add_moves_from(&mut moves, rmoves, from);
        }

        let mut bishop_bb = us.bishop;
        while bishop_bb != 0 {
            let piece = bit_pop(&mut bishop_bb);
            let from = piece.trailing_zeros();

            let attacks = get_line_attacks(occ, diag(from), piece) |
                          get_line_attacks(occ, a_diag(from), piece);

            let bmoves = attacks & !us.pieces;
            add_moves_from(&mut moves, bmoves, from);
        }

        let mut knight_bb = us.knight;
        while knight_bb != 0 {
            let piece = bit_pop(&mut knight_bb);
            let from = piece.trailing_zeros();

            let attacks = KNIGHT_MAP[from as usize];
            let nmoves = attacks & !us.pieces;
            add_moves_from(&mut moves, nmoves, from);
        }

        let mut king_bb = us.king;
        let piece = bit_pop(&mut king_bb);
        let from = piece.trailing_zeros();

        let attacks = KING_MAP[from as usize];
        let kmoves = attacks & !us.pieces;
        add_moves_from(&mut moves, kmoves, from);

        moves
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

        Board { w: w, b: b, sqs: sqs }
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
        write!(f, "--------\n{}--------", output)
    }
}

#[derive(Debug, Default)]
pub struct BitBoard {
    pub pawn: u64,
    pub knight: u64,
    pub bishop: u64,
    pub rook: u64,
    pub queen: u64,
    pub king: u64,
    pub pieces: u64
}