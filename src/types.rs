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

#[derive(Copy, Clone)]
pub struct Move { data: u32 }

impl Move {
    pub fn new(from: u32, to: u32, flags: u32) -> Move {
        let d = from | to << 6 | flags << 12;
        Move { data: d }
    }

    pub fn from(&self)  -> u32 { self.data & 0x3F }
    pub fn to(&self)    -> u32 { (self.data >> 6) & 0x3F }
    pub fn promotion(&self) -> u32 { self.data & 1 }
    // pub fn capture(&self) -> u32
    // pub fn piece(&self) -> u32
}

pub fn str_to_move(mv: &str) -> Move {
    let moves: Vec<char> = mv.chars().collect();
    match moves.as_slice() {
        [sc, sr, dc, dr, promotion..] => {
            let flags = if promotion.len() == 1 { }
        }
        // [sc, sr, dc, dr] => {
        //     self.make_move(Move::new(to_pos(sc, sr), to_pos(dc, dr), 0));
        // },
        // [sc, sr, dc, dr, promotion] => {
        //     let prom_piece = to_piece(if self.move_num % 2 == 1 {promotion.to_ascii_uppercase()} else {promotion});
        //     self.make_promotion(Move::new(to_pos(sc, sr), to_pos(dc, dr), 0), prom_piece); // TODO:
        // }
        _ => () // malformed move
    }
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
