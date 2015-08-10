use std::ascii::AsciiExt;
use std::fmt;
use std::ops::{Index, IndexMut};

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BitBoard(pub [u64; 14]);

impl BitBoard {
    pub fn set_all(&mut self) {
        self[ALL | WHITE] = self[PAWN | WHITE] | self[KNIGHT | WHITE] | self[BISHOP | WHITE] |
                            self[ROOK | WHITE] | self[QUEEN | WHITE]  | self[KING | WHITE];

        self[ALL | BLACK] = self[PAWN | BLACK] | self[KNIGHT | BLACK] | self[BISHOP | BLACK] |
                            self[ROOK | BLACK] | self[QUEEN | BLACK]  | self[KING | BLACK];
    }
}

impl Index<u8> for BitBoard {
    type Output = u64;

    fn index<'a>(&'a self, index: u8) -> &'a u64 {
        &self.0[index as usize]
    }
}

impl IndexMut<u8> for BitBoard {
    fn index_mut<'a>(&'a mut self, index: u8) -> &'a mut u64 {
        &mut self.0[index as usize]
    }
}

pub type Squares = [u8; 64];

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Hash { pub val: u64 }

#[derive(Copy)]
pub struct Board {
    pub bb: BitBoard,
    pub sqs: Squares,
    pub move_num: u32,
    pub hash: Hash,
    pub castling: u8,
    pub en_passant: u64
}

impl Clone for Board { fn clone(&self) -> Self { *self } }

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut characters = Vec::with_capacity(64+8);

        for r in (0..8).rev() {
            for c in 0..8 {
                characters.push(to_char(self.sqs[r*8 + c]));
            }
            characters.push('\n');
        }

        let output: String = characters.into_iter().collect();
        write!(f, "--------\n{}--------\n\
                  Move # {}\n\
                  en passant {}\n\
                  castling {}\n\
                  hash {}\n",
                  output, self.move_num, self.en_passant, self.castling, self.hash.val)
    }
}

pub enum Castle {
    WKing = 1, WQueen = 2, BKing = 4, BQueen = 8
}

// TODO: Index Bitboard with piece type
pub const PAWN: u8   = 0b0000;
pub const KNIGHT: u8 = 0b0010;
pub const BISHOP: u8 = 0b0100;
pub const ROOK: u8   = 0b0110;
pub const QUEEN: u8  = 0b1000;
pub const KING: u8   = 0b1010;
pub const ALL: u8    = 0b1100; // TODO: Rename to ANY?
pub const EMPTY: u8  = 255;

pub const COLOR: u8 = 1;
pub const WHITE: u8 = COLOR;
pub const BLACK: u8 = 0;
pub const PIECE: u8 = 0b1110;

pub fn flip(c: u8) -> u8 {
    !c & COLOR
}

pub const PVALS: [u32; 12] = [1000, 1000,
                              4126, 4126,
                              4222, 4222,
                              6414, 6414,
                              12730, 12730,
                              300000, 300000];

pub fn p_val(piece: u8) -> u32 {
    match piece {
        EMPTY => 0,
        _     => PVALS[piece as usize]
    }
}

pub fn to_piece(c: char) -> u8 {
    let pt = match c.to_ascii_lowercase() {
        'p' => PAWN,
        'n' => KNIGHT,
        'b' => BISHOP,
        'r' => ROOK,
        'q' => QUEEN,
        'k' => KING,
        _   => return EMPTY
    };

    let color = if c.is_uppercase() { WHITE } else { BLACK };
    pt | color
}

pub fn to_char(sq: u8) -> char {
    let ch = match sq & PIECE {
        PAWN   => 'p',
        KNIGHT => 'n',
        BISHOP => 'b',
        ROOK   => 'r',
        QUEEN  => 'q',
        KING   => 'k',
        _      => ' '
    };
    if sq & COLOR == WHITE { ch.to_ascii_uppercase() } else { ch }
}

pub fn to_pos(col: char, row: char) -> u32 {
    let col_num = col as u8 - b'a';
    let row_num = row as u8 - b'1';
    (row_num * 8 + col_num) as u32
}

pub fn from_pos(pos: u32) -> (char, char) {
    let (row, col) = (pos / 8, pos % 8);
    ((col as u8 + b'a') as char, (row as u8 + b'1') as char)
}

pub const KNIGHT_PROM: u32 = 1;
pub const BISHOP_PROM: u32 = 2;
pub const ROOK_PROM: u32   = 3;
pub const QUEEN_PROM: u32  = 4;

pub const CASTLE_KING: u32  = 1 << 3;
pub const CASTLE_QUEEN: u32 = 1 << 4;

pub const IS_CAPTURE: u32 = 1 << 5;
pub const DOUBLE_PAWN_PUSH: u32 = 1 << 6;
pub const EN_PASSANT: u32 = 1 << 7;

// TODO: Consider making move non-copyable, overhead would be lower than passing pointer
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Move { data: u32 }

impl Move {
    pub const fn new(from: u32, to: u32, flags: u32) -> Move {
        Move { data: from | to << 6 | flags << 12 }
    }

    pub fn from(&self)  -> u32 { self.data & 0x3F }
    pub fn to(&self)    -> u32 { (self.data >> 6) & 0x3F }
    pub fn flags(&self) -> u32 { self.data >> 12 }
    pub fn promotion(&self) -> u32 { self.flags() & 0x7 }
    pub fn king_castle(&self) -> bool { self.flags() & CASTLE_KING != 0 }
    pub fn queen_castle(&self) -> bool { self.flags() & CASTLE_QUEEN != 0 }
    pub fn is_capture(&self) -> bool { self.flags() & IS_CAPTURE != 0 }
    pub fn is_double_push(&self) -> bool { self.flags() & DOUBLE_PAWN_PUSH != 0 }
    pub fn is_en_passant(&self) -> bool { self.flags() & EN_PASSANT != 0 }

    pub fn to_str(&self) -> String {
        let (sc, sr) = from_pos(self.from());
        let (dc, dr) = from_pos(self.to());
        let mut chars = vec![sc, sr, dc, dr];

        match self.promotion() {
            KNIGHT_PROM => chars.push('n'),
            BISHOP_PROM => chars.push('b'),
            ROOK_PROM   => chars.push('r'),
            QUEEN_PROM  => chars.push('q'),
            _ => ()
        }

        chars.into_iter().collect::<String>()
    }

    pub const NULL: Move = Move::new(0,0,0);
}
