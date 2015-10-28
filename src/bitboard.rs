use std::ops::{Index, IndexMut};

use board::Squares;
use types::*;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct BitBoard(pub [u64; 14]);

impl BitBoard {
    pub fn generate_from(sqs: &Squares) -> BitBoard {
        let mut bb = BitBoard([0; 14]);

        for (pos, &piece) in sqs.iter().enumerate() {
            if piece != EMPTY { bb[piece] |= 1 << pos }
        }

        bb.set_all();
        bb
    }

    /// Populate the bitboard entries for occupancy
    pub fn set_all(&mut self) {
        self[ALL | WHITE] = self[PAWN | WHITE] | self[KNIGHT | WHITE] | self[BISHOP | WHITE] |
                            self[ROOK | WHITE] | self[QUEEN | WHITE]  | self[KING | WHITE];

        self[ALL | BLACK] = self[PAWN | BLACK] | self[KNIGHT | BLACK] | self[BISHOP | BLACK] |
                            self[ROOK | BLACK] | self[QUEEN | BLACK]  | self[KING | BLACK];
    }
}

impl Index<u8> for BitBoard {
    type Output = u64;

    fn index(&self, index: u8) -> &u64 {
        &self.0[index as usize]
    }
}

impl IndexMut<u8> for BitBoard {
    fn index_mut(&mut self, index: u8) -> &mut u64 {
        &mut self.0[index as usize]
    }
}
