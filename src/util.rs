pub const ROW_1: u64 = 0x00000000000000FF;
pub const ROW_2: u64 = ROW_1 << 8;
pub const ROW_3: u64 = ROW_1 << 8 * 2;
pub const ROW_4: u64 = ROW_1 << 8 * 3;
pub const ROW_5: u64 = ROW_1 << 8 * 4;
pub const ROW_6: u64 = ROW_1 << 8 * 5;
pub const ROW_7: u64 = ROW_1 << 8 * 6;
pub const ROW_8: u64 = ROW_1 << 8 * 7;

pub const FILE_A: u64 = 0x0101010101010101;
pub const FILE_B: u64 = FILE_A << 1;
pub const FILE_C: u64 = FILE_A << 2;
pub const FILE_D: u64 = FILE_A << 3;
pub const FILE_E: u64 = FILE_A << 4;
pub const FILE_F: u64 = FILE_A << 5;
pub const FILE_G: u64 = FILE_A << 6;
pub const FILE_H: u64 = FILE_A << 7;

// (rank_3, rank_8, l_file, r_file, up, left, right)
pub const PAWN_WHITE: (u64, u64, u64, u64, i32, i32, i32) = (ROW_3, ROW_8, FILE_A, FILE_H, 8, 7, 9);
pub const PAWN_BLACK: (u64, u64, u64, u64, i32, i32, i32) = (ROW_6, ROW_1, FILE_H, FILE_A, -8, -9, -7);

pub fn reverse(mut v: u64) -> u64 {
    v = ((v >> 1)  & 0x5555555555555555) | ((v & 0x5555555555555555) << 1);
    v = ((v >> 2)  & 0x3333333333333333) | ((v & 0x3333333333333333) << 2);
    v = ((v >> 4)  & 0x0F0F0F0F0F0F0F0F) | ((v & 0x0F0F0F0F0F0F0F0F) << 4);
    v = ((v >> 8)  & 0x00FF00FF00FF00FF) | ((v & 0x00FF00FF00FF00FF) << 8);
    v = ((v >> 16) & 0x0000FFFF0000FFFF) | ((v & 0x0000FFFF0000FFFF) << 16);
        ((v >> 32) & 0x00000000FFFFFFFF) | ((v & 0x00000000FFFFFFFF) << 32)
}

pub fn get_attacks(piece: u64, occ: u64, mask: u64) -> u64 {
    let pot_blockers = occ & mask;
    let forward = pot_blockers - 2*piece;
    let rev = reverse(reverse(pot_blockers) - 2*reverse(piece));
    (forward ^ rev) & mask
}

pub fn rook_attacks(piece: u64, from: u32, occ: u64) -> u64 {
    get_attacks(piece, occ, file(from)) |
    get_attacks(piece, occ, row(from))
}

pub fn bishop_attacks(piece: u64, from: u32, occ: u64) -> u64 {
    get_attacks(piece, occ, diag(from)) |
    get_attacks(piece, occ, a_diag(from))
}

pub fn for_all(mut pieces: u64, do_work: &mut FnMut(u32)) {
    while pieces != 0 {
        let from = bit_pop(&mut pieces);

        do_work(from);
    }
}

pub fn lsb(val: u64) -> u32 {
    val.trailing_zeros()
}

// #[inline] pub fn bit_pop(x: &mut u64) -> u64 {
//     let lsb = *x & -(*x);
//     // TODO: v & (!v + 1)
//     *x ^= lsb;
//     lsb
// }

// TODO: Change to bit_pop, *b - 1 instead of ^= 1 << lsb
#[inline] pub fn bit_pop(x: &mut u64) -> u32 {
    let lsb_pos = x.trailing_zeros();
    *x ^= 1 << lsb_pos;
    lsb_pos
}

pub fn file(from: u32) -> u64 {
    FILE_A << (from % 8)
}

pub fn row(from: u32) -> u64 {
    ROW_1 << (8 * (from / 8))
}

pub const MAIN_DIAG: u64 = 0x8040201008040201;

pub fn diag(from: u32) -> u64 {
    let diag_index = ((from / 8) - (from % 8)) & 15;
    if diag_index <= 7 {MAIN_DIAG << 8*diag_index} else {MAIN_DIAG >> 8*(16 - diag_index)}
}

pub const MAIN_ANTI_DIAG: u64 = 0x0102040810204080;

pub fn a_diag(from: u32) -> u64 {
    let diag_index = ((from / 8) + (from % 8)) ^ 7;
    if diag_index <= 7 {MAIN_ANTI_DIAG >> 8*diag_index} else {MAIN_ANTI_DIAG << 8*(16-diag_index)}
}
