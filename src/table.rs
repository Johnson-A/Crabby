use rand::{Rng, ThreadRng, thread_rng};
use types::*;

static mut piece_keys: [u64; 64*6*2] = [0; 64*6*2];
static mut castle_keys: [u64; 16] = [0; 16];
static mut ep_keys: [u64; 8] = [0; 8];
static mut color_key: u64 = 0;

fn set_random(arr: &mut [u64], rng: &mut ThreadRng) {
    for elem in arr.iter_mut() {
        *elem = rng.gen();
    }
}

pub unsafe fn init() {
    let mut rng = thread_rng();
    set_random(&mut piece_keys,  &mut rng);
    set_random(&mut castle_keys, &mut rng);
    set_random(&mut ep_keys,     &mut rng);
    color_key = rng.gen();
}

impl Hash {
    pub fn init(board: &Board) -> Hash {
        let mut hash = Hash { val: 0 };

        for (i, sq) in board.sqs.iter().enumerate() {
            if *sq != EMPTY {
                hash.set_piece(i, *sq);
            }
        }

        hash.set_castling(board.castling);
        hash.set_ep(board.en_passant);
        if board.is_white() { hash.flip_color() };

        hash
    }

    fn set_piece(&mut self, pos: usize, sq: u8) {
        let index = pos + (sq & PIECE) as usize * 64 + ((sq & COLOR) / COLOR) as usize * 384;
        self.val ^= unsafe { piece_keys[index] };
    }

    fn set_castling(&mut self, castling: u8) {
        self.val ^= unsafe { castle_keys[castling as usize] };
    }

    fn set_ep(&mut self, en_passant: u64) {
        let file = en_passant.trailing_zeros() % 8;
        self.val ^= unsafe { ep_keys[file as usize] };
    }

    fn flip_color(&mut self) {
        self.val ^= unsafe { color_key };
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum NodeBound { Exact, Alpha, Beta }

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Entry {
    pub hash: Hash,
    pub score: i32,
    pub best_move: Move,
    pub depth: u8,
    pub bound: NodeBound,
    pub ancient: bool
}

impl Entry {
    pub fn is_empty(&self) -> bool {
        *self == Entry::NULL
    }

    const NULL: Entry = Entry { hash: Hash { val: 0 }, score: 0, best_move: Move::NULL,
                                depth: 0, bound: NodeBound::Exact, ancient: false };
}

pub struct Table {
    table: Vec<Entry>
}

impl Table {
    pub fn empty(size: usize) -> Self {
        Table { table: vec![Entry::NULL; size] }
    }

    pub fn index(&self, hash: Hash) -> usize {
        hash.val as usize % self.table.len()
    }

    pub fn probe(&self, hash: Hash, depth: u8, alpha: i32, beta: i32) -> (Option<i32>, Move) {
        let entry = &self.table[self.index(hash)];

        if !entry.is_empty() && entry.hash == hash {
            if entry.depth >= depth {
                match entry.bound {
                    NodeBound::Alpha => if alpha >= entry.score { return (Some(entry.score), Move::NULL) },
                    NodeBound::Beta  => if beta  <= entry.score { return (Some(entry.score), Move::NULL) },
                    NodeBound::Exact => return (Some(entry.score), Move::NULL)
                }
            }
            return (None, entry.best_move)
        }

        (None, Move::NULL)
    }

    pub fn best_move(&self, hash: Hash) -> Option<Move> {
        let entry = &self.table[self.index(hash)];

        if !entry.is_empty() && entry.hash == hash {
            return if entry.best_move == Move::NULL { None } else { Some(entry.best_move) }
            // match entry.best_move {
            //     Move::NULL => return None,
            //     _ => return Some(entry.best_move)
            // }
        }
        None
    }

    pub fn record(&mut self, board: &Board, score: i32, best_move: Move, depth: u8, bound: NodeBound) {
        let ind = self.index(board.hash);
        let entry= &mut self.table[ind];

        if entry.is_empty() || entry.depth > depth || entry.ancient { // TODO: use ancient here
            *entry = Entry { hash: board.hash, score: score, best_move: best_move,
                             depth: depth, bound: bound, ancient: false };
        }
    }
}
