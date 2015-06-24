use std::ascii::AsciiExt;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PieceType { Pawn = 0, Knight = 1, Bishop = 2, Rook = 3, Queen = 4, King = 5 }

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Color { Black = -1, White = 1 }

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Square {
    Empty,
    Piece(PieceType, Color)
}

pub type Squares = [Square; 64];

pub fn to_piece(c: char) -> Square {
    let pt = match c.to_ascii_lowercase() {
        'p' => PieceType::Pawn,
        'n' => PieceType::Knight,
        'b' => PieceType::Bishop,
        'r' => PieceType::Rook,
        'q' => PieceType::Queen,
        'k' => PieceType::King,
        _   => return Square::Empty
    };

    let color = if c.is_lowercase() { Color::Black } else { Color::White };
    Square::Piece(pt, color)
}

pub fn to_char(sq: &Square) -> char {
    match *sq {
        Square::Empty => ' ',
        Square::Piece(pt, color) => {
            let ch = match pt {
                PieceType::Pawn   => 'p',
                PieceType::Knight => 'n',
                PieceType::Bishop => 'b',
                PieceType::Rook   => 'r',
                PieceType::Queen  => 'q',
                PieceType::King   => 'k'
            };
            if color == Color::White { ch.to_ascii_uppercase() } else { ch }
        }
    }
}

pub fn to_pos(col: char, row: char) -> u32 {
    let col_num = col as u8 - b'a';
    let row_num = row as u8 - b'1';
    (row_num * 8 + col_num) as u32
}

pub const EMPTY: u8  = 0;
pub const PAWN: u8   = 1;
pub const KNIGHT: u8 = 2;
pub const BISHOP: u8 = 3;
pub const ROOK: u8   = 4;
pub const QUEEN: u8  = 5;

pub const KNIGHT_PROM: u8 = 1 << 12;
pub const BISHOP_PROM: u8 = 2 << 12;
pub const ROOK_PROM: u8   = 3 << 12;
pub const QUEEN_PROM: u8  = 4 << 12;

pub const CASTLE_KING: u8  = 1 << 15;
pub const CASTLE_QUEEN: u8 = 1 << 16;

#[derive(Copy, Clone)]
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
    pub fn promotion(&self) -> u32 { (self.data >> 12) & 0x7 }

    pub fn to_str(&self) -> String {
        let (from, to) = (self.from() as u8, self.to() as u8);
        let (sr, sc) = (from / 8, from % 8);
        let (dr, dc) = (to / 8, to % 8);
        let (sr_char, sc_char) = ((sr + b'1') as char, (sc + b'a') as char);
        let (dr_char, dc_char) = ((dr + b'1') as char, (dc + b'a') as char);
        let chars = vec![sc_char, sr_char, dc_char, dr_char];
        chars.into_iter().collect::<String>()
    }
}
