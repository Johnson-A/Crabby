#![feature(slice_patterns, core, test, associated_consts)]
extern crate core;
extern crate test;
extern crate time;
extern crate rand;

pub mod board;
pub mod evaluation;
pub mod magics;
pub mod search;
pub mod table;
pub mod testing;
pub mod timer;
pub mod types;
pub mod uci;
pub mod util;

pub fn main() {
    uci::init();
    uci::main_loop();
}
