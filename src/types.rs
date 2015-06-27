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
    pub move_num: u8, // TODO: UGLY hack for now
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

        for (i, sq) in self.sqs.iter().enumerate() {
            characters.push(to_char(*sq));
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
    let ch = match sq & NO_COLOR {
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

pub const PAWN: u8   = 0;
pub const KNIGHT: u8 = 1;
pub const BISHOP: u8 = 2;
pub const ROOK: u8   = 3;
pub const QUEEN: u8  = 4;
pub const KING: u8   = 5;
pub const EMPTY: u8  = 6;

pub const PIECE: u8 = 0x7;
pub const COLOR: u8 = 0x8;
pub const NO_COLOR: u8 = !COLOR;
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

#[derive(Copy, Clone, PartialEq)]
pub struct Move { data: u32 }

impl Move {
    pub fn new(from: u32, to: u32, flags: u32) -> Move {
        let d = from | to << 6 | flags << 12;
        Move { data: d }
    }

    pub const NULL_MOVE: Move = Move { data: 0 };

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
        // TODO: add promotion moves
        let (from, to) = (self.from() as u8, self.to() as u8);
        let (sr, sc) = (from / 8, from % 8);
        let (dr, dc) = (to / 8, to % 8);
        let (sr_char, sc_char) = ((sr + b'1') as char, (sc + b'a') as char);
        let (dr_char, dc_char) = ((dr + b'1') as char, (dc + b'a') as char);
        let chars = vec![sc_char, sr_char, dc_char, dr_char];
        chars.into_iter().collect::<String>()
    }
}
