use std::str::FromStr;
use std::thread::JoinHandle;

pub const ROW_1: u64 = 0xFF;
pub const ROW_2: u64 = ROW_1 << 8;
pub const ROW_3: u64 = ROW_1 << (8 * 2);
pub const ROW_4: u64 = ROW_1 << (8 * 3);
pub const ROW_5: u64 = ROW_1 << (8 * 4);
pub const ROW_6: u64 = ROW_1 << (8 * 5);
pub const ROW_7: u64 = ROW_1 << (8 * 6);
pub const ROW_8: u64 = ROW_1 << (8 * 7);

pub const FILE_A: u64 = 0x0101010101010101;
pub const FILE_B: u64 = FILE_A << 1;
pub const FILE_C: u64 = FILE_A << 2;
pub const FILE_D: u64 = FILE_A << 3;
pub const FILE_E: u64 = FILE_A << 4;
pub const FILE_F: u64 = FILE_A << 5;
pub const FILE_G: u64 = FILE_A << 6;
pub const FILE_H: u64 = FILE_A << 7;

pub fn file(from: u32) -> u64 {
    FILE_A << (from % 8)
}

pub fn row(from: u32) -> u64 {
    ROW_1 << (8 * (from / 8))
}

pub const MAIN_DIAG: u64 = 0x8040201008040201;

pub fn diag(from: u32) -> u64 {
    let diag_index = ((from / 8) - (from % 8)) & 15;
    if diag_index <= 7 {
        MAIN_DIAG << (8 * diag_index)
    } else {
        MAIN_DIAG >> (8 * (16 - diag_index))
    }
}

pub const MAIN_ANTI_DIAG: u64 = 0x0102040810204080;

pub fn anti_diag(from: u32) -> u64 {
    let diag_index = ((from / 8) + (from % 8)) ^ 7;
    if diag_index <= 7 {
        MAIN_ANTI_DIAG >> (8 * diag_index)
    } else {
        MAIN_ANTI_DIAG << (8 * (16 - diag_index))
    }
}

// (row_3, row_8, l_file, r_file, up, left, right)
pub const PAWN_INFO_WHITE: (u64, u64, u64, u64, i32, i32, i32) = (ROW_3, ROW_8, FILE_A, FILE_H, 8, 7, 9);
pub const PAWN_INFO_BLACK: (u64, u64, u64, u64, i32, i32, i32) = (ROW_6, ROW_1, FILE_H, FILE_A, -8, -7, -9);

#[inline] pub fn lsb(val: u64) -> u32 {
    val.trailing_zeros()
}

#[inline] pub fn count(val: u64) -> u32 {
    val.count_ones()
}

#[inline] pub fn bit_pop(x: &mut u64) -> u32 {
    let lsb_pos = lsb(*x);
    *x ^= 1 << lsb_pos;
    lsb_pos
}

/// Reverse the bits in a 64 bit number using a recursive algorithm
/// which swaps the order of sub-elements, starting with even and odd bits
pub fn reverse(mut v: u64) -> u64 {
    v = ((v >> 1)  & 0x5555555555555555) | ((v & 0x5555555555555555) << 1);
    v = ((v >> 2)  & 0x3333333333333333) | ((v & 0x3333333333333333) << 2);
    v = ((v >> 4)  & 0x0F0F0F0F0F0F0F0F) | ((v & 0x0F0F0F0F0F0F0F0F) << 4);
    v = ((v >> 8)  & 0x00FF00FF00FF00FF) | ((v & 0x00FF00FF00FF00FF) << 8);
    v = ((v >> 16) & 0x0000FFFF0000FFFF) | ((v & 0x0000FFFF0000FFFF) << 16);
        ((v >> 32) & 0x00000000FFFFFFFF) | ((v & 0x00000000FFFFFFFF) << 32)
}

/// Calculate sliding piece moves for a given occupancy and mask
pub fn get_attacks(piece: u64, occ: u64, mask: u64) -> u64 {
    let pot_blockers = occ & mask;
    let forward = pot_blockers.wrapping_sub(piece.wrapping_mul(2));
    let rev = reverse(reverse(pot_blockers).wrapping_sub(reverse(piece).wrapping_mul(2)));
    (forward ^ rev) & mask
}

pub fn rook_attacks(piece: u64, from: u32, occ: u64) -> u64 {
    get_attacks(piece, occ, file(from)) |
    get_attacks(piece, occ, row(from))
}

pub fn bishop_attacks(piece: u64, from: u32, occ: u64) -> u64 {
    get_attacks(piece, occ, diag(from)) |
    get_attacks(piece, occ, anti_diag(from))
}

pub fn for_all(mut pieces: u64, do_work: &mut FnMut(u32)) {
    while pieces != 0 {
        let from = bit_pop(&mut pieces);

        do_work(from);
    }
}

macro_rules! lock {
    ($e:expr) => ($e.lock().unwrap());
}

pub fn finish<T>(task: &mut Option<JoinHandle<T>>) -> bool {
    match task.take() {
        Some(unfinished) => unfinished.join().is_ok(),
        None => false
    }
}

pub fn parse<T: FromStr>(p: Option<&str>) -> T {
    p.and_then(|t| t.parse().ok()).expect(&*format!("Could not parse {:?}", p))
}

pub fn parse_or<T: FromStr>(p: Option<&str>, def: T) -> T {
    p.and_then(|t| t.parse().ok()).unwrap_or(def)
}

pub fn try_parse<T: FromStr>(p: Option<&str>) -> Result<T, String> {
    p.ok_or("Value is None".into()).and_then(|t| t.parse().map_err(|_|
        format!("{} cannot be parsed", t)
    ))
}
