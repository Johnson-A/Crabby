use rand::{Rng, thread_rng};
use types::{BISHOP, ROOK};
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

    let size  = get_piece_map(BISHOP, &mut BISHOP_MAP, 0, false);
    let total = get_piece_map(ROOK, &mut ROOK_MAP, size, false);
    assert!(total == MAP_SIZE);
}

pub unsafe fn knight_map_init() {
    let offsets = vec![
    (-1, -2), (-2, -1), (-2, 1), (-1, 2),
    (1, -2),  (2, -1),  (2, 1),  (1, 2)];

    for (i, attacks) in KNIGHT_MAP.iter_mut().enumerate() {
        let (r, c) = ((i / 8) as isize, (i % 8) as isize);

        for &(dr, dc) in &offsets {
            if r+dr >= 0 && c+dc >= 0 && r+dr < 8 && c+dc < 8 {
                *attacks |= 1 << ((r + dr)*8 + c + dc);
            }
        }
    }
}

pub unsafe fn king_map_init() {
    let offsets = vec![
    (1, -1), (1, 0),  (1, 1),
    (0, -1),          (0, 1),
    (-1,-1), (-1, 0), (-1, 1)];

    for (i, attacks) in KING_MAP.iter_mut().enumerate() {
        let (r, c) = ((i / 8) as isize, (i % 8) as isize);

        for &(dr, dc) in &offsets {
            if r+dr >= 0 && c+dc >= 0 && r+dr < 8 && c+dc < 8 {
                *attacks |= 1 << ((r + dr)*8 + c + dc);
            }
        }
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

pub unsafe fn get_piece_map(piece: u8, piece_map: &mut [SMagic; 64],
                            mut offset: usize, from_scratch: bool) -> usize {

    let mut rng = thread_rng();

    for (pos, entry) in piece_map.iter_mut().enumerate() {
        let s = pos as u32;

        let edges = ((ROW_1  | ROW_8)  & !row(s)) |
                    ((FILE_A | FILE_H) & !file(s));

        // The mask for square 's' is the set of moves on an empty board
        let attacks: fn(u64, u32, u64) -> u64 = if piece == BISHOP { bishop_attacks } else { rook_attacks };
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

        let mut magic = if piece == BISHOP { BISHOP_MAGICS[pos] } else { ROOK_MAGICS[pos] };

        'outer: loop {
            if from_scratch { // Generate a new random magic from scratch
                loop {
                    magic = rng.gen::<u64>() & rng.gen::<u64>() & rng.gen::<u64>();
                    if ((magic * mask) >> 56).count_ones() >= 6 { break }
                }
            }

            let mut attacks = vec![0; size];

            for i in 0..size {
                let index = (magic * occupancy[i]) >> shift;
                let attack = &mut attacks[index as usize];

                if *attack != 0 && *attack != reference[i] {
                    assert!(from_scratch, "Error: Precalculated magic is incorrect. Square {}, for {} magic",
                                           pos, if piece == BISHOP { "bishop" } else { "rook" } );
                    continue 'outer
                }

                *attack = reference[i];
            }

            *entry = SMagic { offset: offset, mask: mask, magic: magic, shift: shift };
            for (i, &att) in attacks.iter().enumerate() {
                MAP[offset + i] = att;
            }
            offset += size;

            break // If we've reached this point, all from 0..size have been verified
        }
    }
    offset
}

static BISHOP_MAGICS: [u64; 64] =
[306397059236266368, 6638343277122827280, 10377420549504106496, 9193021019258913, 2306408226914042898, 10379110636817760276, 27167319028441088, 7566153073497751552,
1513227076520969216, 301917653126479936, 72075465430409232, 2343002121441460228, 36033212782477344, 9223373154083475456, 6935629192638251008, 72621648200664064,
2310506081245267984, 2533291987569153, 146934404644733024, 1838417834950912, 579856052833622016, 1729946448243595776, 705208029025040, 2886877732040869888,
10092575566416331020, 5635409948247040, 738739924278198804, 4648849515743289408, 9233786889293807616, 1155253577929753088, 435164712050360592, 3026700562025580641,
4612284839965491969, 10448650511900137472, 571823356120080, 40569782189687936, 148620986995048708, 4901113822871308288, 4612077461748908288, 10204585674276944,
2534512027246592, 5766297627561820676, 13809969191200768, 1153062656578422784, 9318235838682899712, 11533824475839595776, 433770548762247233, 92326036501692936,
9227053213059129360, 577024872779350852, 108087561569959936, 582151826703646856, 81404176367767, 316415319130374273, 9113856212762624, 145453328103440392,
441392350330618400, 1126492748710916, 2309220790581891072, 3026423624667006980, 18019391702696464, 4516931289817600, 1450317422841301124, 9246488805123342592];

static ROOK_MAGICS: [u64; 64] =
[36028867955671040, 2395917338224361536, 936757656041832464, 648535942831284356, 36037595259731970, 13943151043426386048, 432349966580056576, 4683745813775001856,
1191624314978336800, 4611756662317916160, 4625338105090543616, 140806208356480, 1688987371057664, 9288708641522688, 153403870897537280, 281550411726850,
2401883155071024, 1206964838111645696, 166705754384925184, 36039792408011264, 10376580514281768960, 9148486532465664, 578787319189340418, 398007816633254020,
2341872150903791616, 2314850762536009728, 297238127310798880, 2251868801728768, 2594082183614301184, 820222482337235456, 37717655469424904, 577596144088011012,
1152991874030502016, 3171026856472219648, 20415869351890944, 4611844348286345472, 2455605323386324224, 140754676613632, 1740713828645089416, 58361257132164,
70370893791232, 9227880322828615684, 72092778695295040, 577023839834341392, 4723150143565660416, 563087661073408, 651083773116450, 72128789630550047,
153192758223054976, 869194865525653568, 4972009250306933248, 1031325449119138048, 1297041090863464576, 580401419157405824, 1657992643584, 306245066729521664,
15206439601351819394, 14143290885479661953, 1688988407201810, 18065251325837538, 1152927311403745429, 162411078742050817, 334255838724676, 27323018585852550];
