use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::str::SplitWhitespace;

pub type Params<'a> = SplitWhitespace<'a>;
pub type Flag = Arc<AtomicBool>;

pub const BK_CASTLE: u8 = 1;
pub const WK_CASTLE: u8 = BK_CASTLE << WHITE;

pub const BQ_CASTLE: u8 = 1 << 2;
pub const WQ_CASTLE: u8 = BQ_CASTLE << WHITE;

pub const KING_CASTLE: u8  = WK_CASTLE | BK_CASTLE;
pub const QUEEN_CASTLE: u8 = WQ_CASTLE | BQ_CASTLE;

pub const PAWN: u8   = 0;
pub const KNIGHT: u8 = 1 << 1;
pub const BISHOP: u8 = 2 << 1;
pub const ROOK: u8   = 3 << 1;
pub const QUEEN: u8  = 4 << 1;
pub const KING: u8   = 5 << 1;
pub const ALL: u8    = 6 << 1;
pub const EMPTY: u8  = 255;

pub const COLOR: u8 = 1;
pub const WHITE: u8 = COLOR;
pub const BLACK: u8 = 0;
pub const PIECE: u8 = 0b1110;

pub const I_WHITE: usize = WHITE as usize;
pub const I_BLACK: usize = BLACK as usize;

pub fn flip(c: u8) -> u8 {
    c ^ WHITE
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

pub const KNIGHT_PROM: u32 = 1;
pub const BISHOP_PROM: u32 = 2;
pub const ROOK_PROM: u32   = 3;
pub const QUEEN_PROM: u32  = 4;

pub const CASTLES_KING: u32  = 1 << 3;
pub const CASTLES_QUEEN: u32 = 1 << 4;

pub const IS_CAPTURE: u32 = 1 << 5;
pub const DOUBLE_PAWN_PUSH: u32 = 1 << 6;
pub const EN_PASSANT: u32 = 1 << 7;
