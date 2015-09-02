#![feature(slice_patterns, convert, test, associated_consts)]
extern crate test;
extern crate time;
extern crate rand;

pub mod board;
pub mod evaluation;
pub mod magics;
pub mod search;
pub mod table;
pub mod testing;
pub mod types;
pub mod uci;
pub mod util;

pub fn main() {
    uci::init();
    uci::main_loop();
}
