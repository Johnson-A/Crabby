//! Since move keywork is taken, capitalize to Move
#![allow(non_snake_case)]
use types::*;
use util::*;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Move { data: u32 }

#[derive(Copy, Clone)]
pub struct Killer(pub Move, pub Move);

impl Killer {
    pub fn substitute(&mut self, mv: Move) {
        self.1 = self.0;
        self.0 = mv;
    }
}

impl Move {
    pub fn new(from: u32, to: u32, flags: u32) -> Move {
        Move { data: from | to << 6 | flags << 12 }
    }

    pub const NULL: Move = Move { data: 0 };

    pub fn from(&self)  -> u32 { self.data & 0x3F }
    pub fn to(&self)    -> u32 { (self.data >> 6) & 0x3F }
    pub fn flags(&self) -> u32 { self.data >> 12 }
    pub fn promotion(&self) -> u32 { self.flags() & 0x7 }
    pub fn king_castle(&self) -> bool { self.flags() & CASTLES_KING != 0 }
    pub fn queen_castle(&self) -> bool { self.flags() & CASTLES_QUEEN != 0 }
    pub fn is_capture(&self) -> bool { self.flags() & IS_CAPTURE != 0 }
    pub fn is_double_push(&self) -> bool { self.flags() & DOUBLE_PAWN_PUSH != 0 }
    pub fn is_en_passant(&self) -> bool { self.flags() & EN_PASSANT != 0 }
}

pub fn add_moves(moves: &mut Vec<Move>, mut targets: u64, diff: i32, flags: u32) {
    while targets != 0 {
        let to = bit_pop(&mut targets);
        let from = (to as i32 - diff) as u32;
        moves.push(Move::new(from, to, flags));
    }
}

pub fn add_moves_from(moves: &mut Vec<Move>, from: u32, mut targets: u64, flags: u32) {
    while targets != 0 {
        let to = bit_pop(&mut targets);
        moves.push(Move::new(from, to, flags));
    }
}

pub fn add_prom_moves(moves: &mut Vec<Move>, mut targets: u64, diff: i32, flags: u32) {
    while targets != 0 {
        let to = bit_pop(&mut targets);
        let from = (to as i32 - diff) as u32;

        moves.push(Move::new(from, to, flags | QUEEN_PROM));
        moves.push(Move::new(from, to, flags | ROOK_PROM));
        moves.push(Move::new(from, to, flags | KNIGHT_PROM));
        moves.push(Move::new(from, to, flags | BISHOP_PROM));
    }
}
