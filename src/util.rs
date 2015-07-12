extern crate rand;
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

pub fn for_all_pieces(mut pieces: u64, do_work: &mut FnMut(u32)) {
    while pieces != 0 {
        let from = bit_pop_pos(&mut pieces);

        do_work(from);
    }
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

pub unsafe fn knight_map_init() {
    let offsets = vec![
    (-1, -2), (-2, -1), (-2, 1), (-1, 2),
    (1, -2),  (2, -1),  (2, 1),  (1, 2)];

    for (i, att) in KNIGHT_MAP.iter_mut().enumerate() {
        let mut targets = 0;
        let (r, c) = ((i / 8) as isize, (i % 8) as isize);

        for &(dr, dc) in &offsets {
            if (r+dr >= 0) & (c+dc >= 0) & (r+dr < 8) & (c+dc < 8) {
                targets |= 1 << ((r+dr)*8 + (c+dc));
            }
        }
        *att = targets;
    }
}

pub unsafe fn king_map_init() {
    let offsets = vec![
    (1, -1), (1, 0),  (1, 1),
    (0, -1),          (0, 1),
    (-1,-1), (-1, 0), (-1, 1)];

    for (i, att) in KING_MAP.iter_mut().enumerate() {
        let mut targets = 0;
        let (r, c) = ((i / 8) as isize, (i % 8) as isize);

        for &(dr, dc) in &offsets {
            if (r+dr >= 0) & (c+dc >= 0) & (r+dr < 8) & (c+dc < 8) {
                targets |= 1 << ((r+dr)*8 + (c+dc));
            }
        }
        *att = targets;
    }
}

#[derive(Copy, Clone)]
pub struct SMagic {
    pub offset: usize,
    pub mask: u64,
    pub magic: u64,
    pub shift: u32
}

impl SMagic {
    pub unsafe fn att(&self, occ: u64) -> u64 {
        let ind = (self.magic * (occ & self.mask)) >> self.shift;
        MAP[self.offset + ind as usize]
    }
}


pub unsafe fn init() {
    let mut table = Vec::new();
    king_map_init();
    knight_map_init();
    BISHOP_MAP = get_piece_map(&bishop_attacks, &mut table);
    ROOK_MAP = get_piece_map(&rook_attacks, &mut table);
    for (i, elem) in table.iter().enumerate() {
        MAP[i] = *elem;
    }
}

pub static mut KING_MAP: [u64; 64] = [0; 64];
pub static mut KNIGHT_MAP: [u64; 64] = [0; 64];
pub static mut BISHOP_MAP: [SMagic; 64] = [SMagic { offset: 0, mask: 0, magic: 0, shift: 0 }; 64];
pub static mut ROOK_MAP: [SMagic; 64] = [SMagic { offset: 0, mask: 0, magic: 0, shift: 0 }; 64];
pub static mut MAP: [u64; 107648] = [0; 107648];

pub fn get_piece_map(attacks: &Fn(u64, u32, u64) -> u64, table: &mut Vec<u64>) -> [SMagic; 64] {
    let mut map = [SMagic { offset: 0, mask: 0, magic: 0, shift: 0 }; 64];
    let mut offset = table.len();
    let mut rng = rand::thread_rng();

    for (pos, entry) in map.iter_mut().enumerate() {
        let s = pos as u32;

        let edges = ((ROW_1  | ROW_8)  & !row(s)) |
                    ((FILE_A | FILE_H) & !file(s));

        // The mask for square 's' is the set of moves on an empty board
        let mask = attacks(1 << s, s, 1 << s) & !edges;
        let num_ones = mask.count_ones();
        let shift = 64 - num_ones;

        let mut occupancy = vec![0; 1 << num_ones];
        let mut reference = vec![0; 1 << num_ones];

        let mut size = 0;
        let mut occ = 0;
        loop {
            occupancy[size] = occ;
            reference[size] = attacks(1 << s, s, occ | (1 << s));

            size += 1;
            occ = (occ - mask) & mask;
            if occ == 0 { break } // We will have enumerated all subsets of mask
        }

        'outer: loop {
            let mut magic;
            loop {
                magic = rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>();
                // TODO: Sparse random number
                if ((magic * mask) >> 56).count_ones() >= 6 { break }
            }
            let mut attacks = vec![0; size];

            for i in 0..size {
                let index = (magic * occupancy[i]) >> shift;
                let attack = &mut attacks[index as usize];

                if (*attack != 0) & (*attack != reference[i]) {
                    continue 'outer
                }

                *attack = reference[i];
            }

            *entry = SMagic { offset: offset, mask: mask, magic: magic, shift: shift };
            offset += size;
            table.append(&mut attacks);
            break // If we've reached this point, all from 0..size have been verified
        }
    }
    map
}
