use rand::{Rng, SeedableRng, StdRng};
use std::collections::HashSet;
use std::mem;
use types::*;
use _move::Move;
use board::*;
use util::lsb;

static mut piece_keys: [u64; 64*6*2] = [0; 64*6*2];
static mut castle_keys: [u64; 16] = [0; 16];
static mut ep_keys: [u64; 8] = [0; 8];
static mut color_key: u64 = 0;

fn set_random(arr: &mut [u64], rng: &mut StdRng) {
    for elem in arr.iter_mut() {
        *elem = rng.gen();
    }
}

pub unsafe fn init() {
    let seed: &[usize] = &[0];
    let rng = &mut SeedableRng::from_seed(seed);
    set_random(&mut piece_keys,  rng);
    set_random(&mut castle_keys, rng);
    set_random(&mut ep_keys,     rng);
    color_key = rng.gen();
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Hash { pub val: u64 }

impl Hash {
    pub fn init(sqs: &Squares, castling: u8, en_passant: u64, color: u8) -> Self {
        let mut hash = Hash { val: 0 };

        for (i, &sq) in sqs.iter().enumerate() {
            hash.set_piece(i, sq);
        }

        hash.set_castling(castling);
        hash.set_ep(en_passant);
        if color == WHITE { hash.flip_color() };
        hash
    }

    pub fn set_piece(&mut self, pos: usize, sq: u8) {
        if sq != EMPTY {
            let index = pos + ((sq & PIECE) >> 1) as usize * 64 + (sq & COLOR) as usize * 384;
            self.val ^= unsafe { piece_keys[index] };
        }
    }

    pub fn set_castling(&mut self, castling: u8) {
        self.val ^= unsafe { castle_keys[castling as usize] };
    }

    pub fn set_ep(&mut self, en_passant: u64) {
        if en_passant != 0 {
            let file = lsb(en_passant) % 8;
            self.val ^= unsafe { ep_keys[file as usize] };
        }
    }

    pub fn flip_color(&mut self) {
        self.val ^= unsafe { color_key };
    }

    pub fn sub(&self) -> u16 {
        (self.val >> 48) as u16
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum Bound { Exact = 0, Lower = 1, Upper = 2 }

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Entry {
    pub score: i32,
    pub best_move: Move,
    pub info: u32
}

impl Entry {
    const NULL: Entry = Entry { score: !0, best_move: Move::NULL, info: !0 };

    pub fn is_empty(&self) -> bool {
        *self == Entry::NULL
    }

    pub fn depth(&self) -> u8 {
        self.info as u8
    }

    pub fn ancient(&self) -> bool {
        self.info & (1 << 8) != 0
    }

    pub fn bound(&self) -> Bound {
        unsafe { mem::transmute(((self.info >> 9) & 0b11) as u8) }
    }

    pub fn compare(&self, hash: &Hash) -> bool {
        (self.info >> 16) as u16 == hash.sub()
    }
}

pub struct Table {
    pub units: Vec<Unit>
}

pub struct Unit {
    entries: [Entry; UNIT_SIZE]
}

impl Clone for Unit { fn clone(&self) -> Self { Unit { entries: self.entries } } }

pub const UNIT_SIZE: usize = 0xFFFF + 1;

impl Table {
    pub fn empty(size: usize) -> Self {
        Table { units: vec![Unit { entries: [Entry::NULL; UNIT_SIZE] }; size / UNIT_SIZE] }
    }

    pub fn empty_mb(size_mb: usize) -> Self {
        Table::empty(size_mb * 1024 * 1024 / mem::size_of::<Entry>())
    }

    pub fn entry(&self, hash: Hash) -> &Entry {
        let unit = &self.units[hash.val as usize % self.units.len()];
        &unit.entries[hash.val as usize % UNIT_SIZE]
    }

    pub fn mut_entry(&mut self, hash: Hash) -> &mut Entry {
        let num_units = self.units.len();
        let unit = &mut self.units[hash.val as usize % num_units];
        &mut unit.entries[hash.val as usize % UNIT_SIZE]
    }

    pub fn probe(&self, hash: Hash, depth: u8, alpha: i32, beta: i32) -> (Option<i32>, Move) {
        let entry = self.entry(hash);

        if !entry.is_empty() && entry.compare(&hash) {
            if  entry.depth() > depth &&
                match entry.bound() {
                    Bound::Lower => entry.score >= beta,
                    Bound::Upper => entry.score <= alpha,
                    Bound::Exact => true }
                { return (Some(entry.score), Move::NULL) }

            return (None, entry.best_move)
        }
        (None, Move::NULL)
    }

    pub fn best_move(&self, hash: Hash) -> Option<Move> {
        let entry = self.entry(hash);

        if !entry.is_empty() && entry.compare(&hash) && entry.best_move != Move::NULL {
            return Some(entry.best_move)
        }
        None
    }

    pub fn record(&mut self, board: &Board, score: i32, best_move: Move, depth: u8, bound: Bound) {
        let entry = self.mut_entry(board.hash);
        let info = depth as u32 | (bound as u32) << 9 | (board.hash.sub() as u32) << 16;
        if entry.is_empty() || entry.depth() <= depth || entry.ancient() {
            *entry = Entry { score: score, best_move: best_move, info: info };
        }
    }

    pub fn pv(&self, board: &Board) -> Vec<Move> {
        let mut pv = Vec::new();
        let mut visited = HashSet::new();
        self.pv_cycle_track(&mut board.clone(), &mut pv, &mut visited);
        pv
    }

    pub fn pv_cycle_track(&self, board: &mut Board, pv: &mut Vec<Move>, visited: &mut HashSet<Hash>) {
        let mv = self.best_move(board.hash);

        match mv {
            Some(m) => {
                pv.push(m);
                board.make_move(m);

                if visited.insert(board.hash) {
                    self.pv_cycle_track(board, pv, visited);
                }
            },
            None => ()
        }
    }

    pub fn num_elems(&self) -> usize {
        self.units.len() * UNIT_SIZE
    }

    pub fn set_ancient(&mut self) -> usize {
        let mut num = 0;
        for unit in &mut self.units {
            for entry in unit.entries.iter_mut() {
                if !entry.is_empty() {
                    num += 1;
                    entry.info |= 1 << 8;
                }
            }
        }
        num
    }
}
