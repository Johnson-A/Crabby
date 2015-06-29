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

// struct SMagic {
//     mask: u64,
//     magic: u64,
//     shift: u32
// }
//
// pub fn mbishop_attacks(mut occ: u64, sq: usize) -> u64 {
//     occ &= BISHOP_TABLE[sq].mask;
//     occ *= BISHOP_TABLE[sq].magic;
//     occ >>= BISHOP_TABLE[sq].shift;
//     ATTACK_TABLE[occ]
// }

#[inline] pub fn reverse(mut v: u64) -> u64 {
    v = ((v >> 1)  & 0x5555555555555555) | ((v & 0x5555555555555555) << 1);
    v = ((v >> 2)  & 0x3333333333333333) | ((v & 0x3333333333333333) << 2);
    v = ((v >> 4)  & 0x0F0F0F0F0F0F0F0F) | ((v & 0x0F0F0F0F0F0F0F0F) << 4);
    v = ((v >> 8)  & 0x00FF00FF00FF00FF) | ((v & 0x00FF00FF00FF00FF) << 8);
    v = ((v >> 16) & 0x0000FFFF0000FFFF) | ((v & 0x0000FFFF0000FFFF) << 16);
        ((v >> 32) & 0x00000000FFFFFFFF) | ((v & 0x00000000FFFFFFFF) << 32)
}

#[inline] pub fn get_attacks(piece: u64, occ: u64, mask: u64) -> u64 {
    let pot_blockers = occ & mask;
    let forward = pot_blockers - 2*piece;
    let rev = reverse(reverse(pot_blockers) - 2*reverse(piece));
    (forward ^ rev) & mask
}

#[inline] pub fn queen_attacks(piece: u64, from: u32, occ: u64) -> u64 {
    get_attacks(piece, occ, file(from)) |
    get_attacks(piece, occ, row(from))  |
    get_attacks(piece, occ, diag(from)) |
    get_attacks(piece, occ, a_diag(from))
}

#[inline] pub fn rook_attacks(piece: u64, from: u32, occ: u64) -> u64 {
    get_attacks(piece, occ, file(from)) |
    get_attacks(piece, occ, row(from))
}

#[inline] pub fn bishop_attacks(piece: u64, from: u32, occ: u64) -> u64 {
    get_attacks(piece, occ, diag(from)) |
    get_attacks(piece, occ, a_diag(from))
}

#[inline] pub fn knight_attacks(from: u32) -> u64 {
    KNIGHT_MAP[from as usize]
}

#[inline] pub fn king_attacks(from: u32) -> u64 {
    KING_MAP[from as usize]
}

#[inline] pub fn bit_pop(x: &mut u64) -> u64 {
    let lsb = *x & -(*x);
    // TODO: v & (!v + 1)
    *x ^= lsb;
    lsb
}

#[inline] pub fn bit_pop_pos(x: &mut u64) -> u32 {
    let lsb_pos = x.trailing_zeros();
    *x ^= 1 << lsb_pos;
    lsb_pos
}

#[inline] pub fn file(from: u32) -> u64 {
    FILE_A << (from % 8)
}

#[inline] pub fn row(from: u32) -> u64 {
    ROW_1 << (8 * (from / 8))
}

pub const MAIN_DIAG: u64 = 0x8040201008040201;

#[inline] pub fn diag(from: u32) -> u64 {
    let diag_index = ((from / 8) - (from % 8)) & 15;
    if diag_index <= 7 {MAIN_DIAG << 8*diag_index} else {MAIN_DIAG >> 8*(16 - diag_index)}
}

pub const MAIN_ANTI_DIAG: u64 = 0x0102040810204080;

#[inline] pub fn a_diag(from: u32) -> u64 {
    let diag_index = ((from / 8) + (from % 8)) ^ 7;
    if diag_index <= 7 {MAIN_ANTI_DIAG >> 8*diag_index} else {MAIN_ANTI_DIAG << 8*(16-diag_index)}
}

lazy_static! {
    pub static ref KNIGHT_MAP: [u64; 64] = {
        let mut map = [0; 64];
        let offsets = vec![
        (-1, -2), (-2, -1), (-2, 1), (-1, 2),
        (1, -2),  (2, -1),  (2, 1),  (1, 2)];

        for (i, att) in map.iter_mut().enumerate() {
            let mut targets = 0;
            let (r, c) = ((i / 8) as isize, (i % 8) as isize);

            for &(dr, dc) in &offsets {
                if (r+dr >= 0) & (c+dc >= 0) & (r+dr < 8) & (c+dc < 8) {
                    targets |= 1 << ((r+dr)*8 + (c+dc));
                }
            }
            *att = targets;
        }
        map
    };

    pub static ref KING_MAP: [u64; 64] = {
        let mut map = [0; 64];
        let offsets = vec![
        (1, -1), (1, 0),  (1, 1),
        (0, -1),          (0, 1),
        (-1,-1), (-1, 0), (-1, 1)];

        for (i, att) in map.iter_mut().enumerate() {
            let mut targets = 0;
            let (r, c) = ((i / 8) as isize, (i % 8) as isize);

            for &(dr, dc) in &offsets {
                if (r+dr >= 0) & (c+dc >= 0) & (r+dr < 8) & (c+dc < 8) {
                    targets |= 1 << ((r+dr)*8 + (c+dc));
                }
            }
            *att = targets;
        }
        map
    };
}
