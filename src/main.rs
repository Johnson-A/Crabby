#![feature(slice_patterns, test, associated_consts)]
extern crate itertools;
extern crate rand;
extern crate time;
extern crate test;

#[macro_use]
pub mod util;

pub mod bitboard;
pub mod board;
pub mod evaluation;
pub mod magics;
pub mod _move;
pub mod print;
pub mod search;
pub mod table;
pub mod testing;
pub mod timer;
pub mod types;
pub mod uci;

pub fn main() {
    uci::main_loop();
}
