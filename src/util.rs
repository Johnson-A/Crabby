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

const BIT_REV: [u64; 256] = [
    0x00, 0x80, 0x40, 0xC0, 0x20, 0xA0, 0x60, 0xE0, 0x10, 0x90, 0x50, 0xD0, 0x30, 0xB0, 0x70, 0xF0,
    0x08, 0x88, 0x48, 0xC8, 0x28, 0xA8, 0x68, 0xE8, 0x18, 0x98, 0x58, 0xD8, 0x38, 0xB8, 0x78, 0xF8,
    0x04, 0x84, 0x44, 0xC4, 0x24, 0xA4, 0x64, 0xE4, 0x14, 0x94, 0x54, 0xD4, 0x34, 0xB4, 0x74, 0xF4,
    0x0C, 0x8C, 0x4C, 0xCC, 0x2C, 0xAC, 0x6C, 0xEC, 0x1C, 0x9C, 0x5C, 0xDC, 0x3C, 0xBC, 0x7C, 0xFC,
    0x02, 0x82, 0x42, 0xC2, 0x22, 0xA2, 0x62, 0xE2, 0x12, 0x92, 0x52, 0xD2, 0x32, 0xB2, 0x72, 0xF2,
    0x0A, 0x8A, 0x4A, 0xCA, 0x2A, 0xAA, 0x6A, 0xEA, 0x1A, 0x9A, 0x5A, 0xDA, 0x3A, 0xBA, 0x7A, 0xFA,
    0x06, 0x86, 0x46, 0xC6, 0x26, 0xA6, 0x66, 0xE6, 0x16, 0x96, 0x56, 0xD6, 0x36, 0xB6, 0x76, 0xF6,
    0x0E, 0x8E, 0x4E, 0xCE, 0x2E, 0xAE, 0x6E, 0xEE, 0x1E, 0x9E, 0x5E, 0xDE, 0x3E, 0xBE, 0x7E, 0xFE,
    0x01, 0x81, 0x41, 0xC1, 0x21, 0xA1, 0x61, 0xE1, 0x11, 0x91, 0x51, 0xD1, 0x31, 0xB1, 0x71, 0xF1,
    0x09, 0x89, 0x49, 0xC9, 0x29, 0xA9, 0x69, 0xE9, 0x19, 0x99, 0x59, 0xD9, 0x39, 0xB9, 0x79, 0xF9,
    0x05, 0x85, 0x45, 0xC5, 0x25, 0xA5, 0x65, 0xE5, 0x15, 0x95, 0x55, 0xD5, 0x35, 0xB5, 0x75, 0xF5,
    0x0D, 0x8D, 0x4D, 0xCD, 0x2D, 0xAD, 0x6D, 0xED, 0x1D, 0x9D, 0x5D, 0xDD, 0x3D, 0xBD, 0x7D, 0xFD,
    0x03, 0x83, 0x43, 0xC3, 0x23, 0xA3, 0x63, 0xE3, 0x13, 0x93, 0x53, 0xD3, 0x33, 0xB3, 0x73, 0xF3,
    0x0B, 0x8B, 0x4B, 0xCB, 0x2B, 0xAB, 0x6B, 0xEB, 0x1B, 0x9B, 0x5B, 0xDB, 0x3B, 0xBB, 0x7B, 0xFB,
    0x07, 0x87, 0x47, 0xC7, 0x27, 0xA7, 0x67, 0xE7, 0x17, 0x97, 0x57, 0xD7, 0x37, 0xB7, 0x77, 0xF7,
    0x0F, 0x8F, 0x4F, 0xCF, 0x2F, 0xAF, 0x6F, 0xEF, 0x1F, 0x9F, 0x5F, 0xDF, 0x3F, 0xBF, 0x7F, 0xFF
];

#[inline] pub fn reverse(bits: u64) -> u64 {
    (BIT_REV[(bits & 0xff) as usize]         << 56) |
    (BIT_REV[((bits >> 8)  & 0xff) as usize] << 48) |
    (BIT_REV[((bits >> 16) & 0xff) as usize] << 40) |
    (BIT_REV[((bits >> 24) & 0xff) as usize] << 32) |
    (BIT_REV[((bits >> 32) & 0xff) as usize] << 24) |
    (BIT_REV[((bits >> 40) & 0xff) as usize] << 16) |
    (BIT_REV[((bits >> 48) & 0xff) as usize] << 8)  |
    BIT_REV[((bits >> 56) & 0xff) as usize]
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
