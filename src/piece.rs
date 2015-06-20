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

pub type Position = (usize, usize);

pub fn to_pos(col: char, row: char) -> Position {
	let col_num = col as u8 - b'a';
	let row_num = row as u8 - b'1';
	(row_num as usize, col_num as usize)
}

pub fn move_to_str(from: u8, to: u8) -> String {
	let (sr, sc) = (from / 8, from % 8);
	let (dr, dc) = (to / 8, to % 8);
	let (sr_char, sc_char) = ((sr + b'1') as char, (sc + b'a') as char);
	let (dr_char, dc_char) = ((dr + b'1') as char, (dc + b'a') as char);
	let chars = vec![sc_char, sr_char, dc_char, dr_char];
	chars.into_iter().collect::<String>()
}