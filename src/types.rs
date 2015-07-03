use std::ascii::AsciiExt;
use std::fmt;

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

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut characters = Vec::with_capacity(64);

        for r in (0..8).rev() {
            for c in 0..8 {
                characters.push(to_char(self.sqs[r*8 + c]));
            }
            characters.push('\n');
        }

        let output = characters.iter().cloned().collect::<String>();
        write!(f, "--------\n{}--------\n\
                  Move # {}\n\
                  wkcas {} wqcas {} bkcas {} bqcas {}\n\
                  en passant {}",
                  output, self.move_num,
                  self.w_k_castle, self.w_q_castle, self.b_k_castle, self.b_q_castle,
                  self.en_passant)
    }
}

pub type Squares = [u8; 64];

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

pub const PAWN: u8   = 0;
pub const KNIGHT: u8 = 1;
pub const BISHOP: u8 = 2;
pub const ROOK: u8   = 3;
pub const QUEEN: u8  = 4;
pub const KING: u8   = 5;
pub const EMPTY: u8  = 6;

pub const PIECE: u8 = 0x7;
pub const COLOR: u8 = 0x8;
pub const WHITE: u8 = COLOR;
pub const BLACK: u8 = 0;

pub const KNIGHT_PROM: u32 = 1;
pub const BISHOP_PROM: u32 = 2;
pub const ROOK_PROM: u32   = 3;
pub const QUEEN_PROM: u32  = 4;

pub const CASTLE_KING: u32  = 1 << 3;
pub const CASTLE_QUEEN: u32 = 1 << 4;

pub const IS_CAPTURE: u32 = 1 << 5;
pub const DOUBLE_PAWN_PUSH: u32 = 1 << 6;
pub const EN_PASSANT: u32 = 1 << 7;

// Make move not copyable
#[derive(Copy, Clone)]
pub struct Move { data: u32 }

impl Move {
    pub fn new(from: u32, to: u32, flags: u32) -> Move {
        let d = from | to << 6 | flags << 12;
        Move { data: d }
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
}
