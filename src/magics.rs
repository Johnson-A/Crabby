use rand::{Rng, thread_rng};
use util::*;

static mut KING_MAP: [u64; 64] = [0; 64];
static mut KNIGHT_MAP: [u64; 64] = [0; 64];
static mut BISHOP_MAP: [SMagic; 64] = [SMagic { offset: 0, mask: 0, magic: 0, shift: 0 }; 64];
static mut ROOK_MAP: [SMagic; 64] = [SMagic { offset: 0, mask: 0, magic: 0, shift: 0 }; 64];

const MAP_SIZE: usize = 107648;
static mut MAP: [u64; MAP_SIZE] = [0; MAP_SIZE];

pub fn knight_moves(from: u32) -> u64 {
    unsafe { KNIGHT_MAP[from as usize] }
}

pub fn king_moves(from: u32) -> u64 {
    unsafe { KING_MAP[from as usize] }
}

pub fn bishop_moves(from: u32, occ: u64) -> u64 {
    unsafe { BISHOP_MAP[from as usize].att(occ) }
}

pub fn rook_moves(from: u32, occ: u64) -> u64 {
    unsafe { ROOK_MAP[from as usize].att(occ) }
}

pub fn queen_moves(from: u32, occ: u64) -> u64 {
    unsafe { BISHOP_MAP[from as usize].att(occ) |
               ROOK_MAP[from as usize].att(occ) }
}

pub unsafe fn init() {
    king_map_init();
    knight_map_init();

    let size  = get_piece_map(&bishop_attacks, &mut BISHOP_MAP, 0);
    let total = get_piece_map(&rook_attacks, &mut ROOK_MAP, size);
    assert!(total == MAP_SIZE)
}

pub unsafe fn knight_map_init() {
    let offsets = vec![
    (-1, -2), (-2, -1), (-2, 1), (-1, 2),
    (1, -2),  (2, -1),  (2, 1),  (1, 2)];

    for (i, att) in KNIGHT_MAP.iter_mut().enumerate() {
        let mut targets = 0;
        let (r, c) = ((i / 8) as isize, (i % 8) as isize);

        for &(dr, dc) in &offsets {
            if r+dr >= 0 && c+dc >= 0 && r+dr < 8 && c+dc < 8 {
                targets |= 1 << ((r + dr)*8 + c + dc);
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
            if r+dr >= 0 && c+dc >= 0 && r+dr < 8 && c+dc < 8 {
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

pub unsafe fn get_piece_map(attacks: &Fn(u64, u32, u64) -> u64, piece_map: &mut [SMagic; 64], mut offset: usize) -> usize {
    let mut rng = thread_rng();

    for (pos, entry) in piece_map.iter_mut().enumerate() {
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
                if ((magic * mask) >> 56).count_ones() >= 6 { break }
            }
            let mut attacks = vec![0; size];

            for i in 0..size {
                let index = (magic * occupancy[i]) >> shift;
                let attack = &mut attacks[index as usize];

                if *attack != 0 && *attack != reference[i] {
                    continue 'outer
                }

                *attack = reference[i];
            }

            *entry = SMagic { offset: offset, mask: mask, magic: magic, shift: shift };
            for i in 0..size {
                MAP[offset + i] = attacks[i];
            }
            offset += size;

            break // If we've reached this point, all from 0..size have been verified
        }
    }
    offset
}
