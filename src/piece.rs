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

impl Color {
    pub fn flip(&mut self) {
        *self = if *self == Color::White {Color::Black} else {Color::White};
    }
}

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
