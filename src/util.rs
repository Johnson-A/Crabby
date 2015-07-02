extern crate rand;
use std::iter::repeat;
use rand::Rng;

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

    pub static ref BISHOP_MAP: PieceMap = get_piece_map(&bishop_attacks);
    pub static ref ROOK_MAP: PieceMap = get_piece_map(&rook_attacks);
}

pub fn get_piece_map(attacks: &Fn(u64, u32, u64) -> u64) -> PieceMap {
    let mut map = repeat(vec![]).take(64).collect::<Vec<_>>();
    let mut masks = [0; 64];
    let mut shifts = [0; 64];
    let mut magics = [0; 64];
    let mut rng = rand::thread_rng();

    for (pos, entry) in map.iter_mut().enumerate() {
        let s = pos as u32;

        let edges = ((ROW_1  | ROW_8)  & !row(s)) |
                    ((FILE_A | FILE_H) & !file(s));

        // The mask for square 's' is the set of moves on an empty board
        masks[pos] = attacks(1 << s, s, 1 << s) & !edges;
        let num_ones = masks[pos].count_ones();
        shifts[pos] = 64 - num_ones;

        let mut occupancy = vec![0; 1 << num_ones];
        let mut reference = vec![0; 1 << num_ones];

        let mut size = 0;
        let mut occ = 0;
        loop {
            occupancy[size] = occ;
            reference[size] = attacks(1 << s, s, occ | (1 << s));

            size += 1;
            occ = (occ - masks[pos]) & masks[pos];
            if occ == 0 { break } // We will have enumerated all subsets of mask
        }

        'outer: loop {
            loop {
                magics[pos] = rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>();
                // TODO: Sparse random number
                if ((magics[pos] * masks[pos]) >> 56).count_ones() >= 6 { break }
            }
            *entry = vec![0; size];

            for i in 0..size {
                let index = (magics[pos] * occupancy[i]) >> shifts[pos];
                let attack = &mut entry[index as usize];

                if (*attack != 0) & (*attack != reference[i]) {
                    continue 'outer
                }

                *attack = reference[i];
            }
            break // If we've reached this point, all from 0..size have been verified
        }
    }
    PieceMap { magics: magics, shifts: shifts, masks: masks, map: map }
}

pub struct PieceMap {
    pub magics: [u64; 64],
    pub shifts: [u32; 64],
    pub masks: [u64; 64],
    pub map: Vec<Vec<u64>>
}

impl PieceMap {
    pub fn att(&self, s: usize, occ: u64) -> u64 {
        let ind = (self.magics[s] * (occ & self.masks[s])) >> self.shifts[s];
        self.map[s][ind as usize]
    }

    pub fn size(&self) -> usize {
        let mut size = 0;
        for sq_map in &self.map {
            size += 8 * sq_map.len();
        }
        size
    }
}
