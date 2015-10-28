//! Handle the transformation from and to strings for associated types
use std::ascii::AsciiExt;
use std::fmt::{Display, Formatter, Result};

use board::Board;
use Move::Move;
use types::*;
use util::*;

impl Display for Board {
    fn fmt(&self, f: &mut Formatter) -> Result {
        let mut characters = Vec::with_capacity(64*2 + 8*2);

        for r in (0..8).rev() {
            characters.push('|');
            for c in 0..8 {
                characters.push(to_char(self.sqs[r*8 + c]));
                characters.push('|');
            }
            characters.push('\n');
        }

        let board: String = characters.into_iter().collect();

        let ep_str = match self.en_passant {
            0 => ('-', '-'),
            _ => from_pos(lsb(self.en_passant))
        };

        write!(f, "-----------------\n\
                  {}\
                  -----------------\n\
                  Move # {}\n\
                  en passant {}{}\n\
                  castling {:04b}\n",
                  board, self.ply + 1, ep_str.0, ep_str.1, self.castling)
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut Formatter) -> Result {
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

        write!(f, "{}", chars.into_iter().collect::<String>())
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
