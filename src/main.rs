//! This is the Crabby chess engine
#![feature(slice_patterns, convert, test, associated_consts, const_fn)]
extern crate test;
extern crate time;
extern crate rand;

use std::io::{stdin, BufReader};
use std::io::prelude::*;
use std::fs::File;

pub mod board;
pub mod evaluation;
pub mod magics;
pub mod search;
pub mod table;
pub mod types;
pub mod util;
pub mod bench;

use types::*;
use search::Searcher;

const ENGINE_NAME: &'static str = "Crabby";

pub fn main() {
    unsafe {
        magics::init();
        table::init();
    }

    let stdin = stdin();
    let mut searcher = Searcher::new_start();

    for line in stdin.lock().lines() {
        let line = line.unwrap_or("".into());
        let mut params = line.split_whitespace();

        if let Some(first_word) = params.next() {
            match first_word {
                "uci"        => uci(),
                "setoption"  => (),
                "isready"    => println!("readyok"),
                "ucinewgame" => searcher = Searcher::new_start(),
                "position"   => searcher.position(&mut params),
                "go"         => searcher.go(&mut params),
                "perft"      => perft(&searcher.root, &mut params),
                "testperf"   => test_positions("testing/positions/performance", &mut searcher, &mut |s| s.id()),
                "testmove"   => test_positions("testing/positions/perftsuite.epd", &mut searcher,
                                                &mut |s| println!("{}", s.root.perft(6, true))),
                "print"      => (),
                _            => println!("Unknown command: {}", first_word)
            }
        }
    }
}

pub fn perft(board: &Board, params: &mut Params) {
    let d = match params.next() {
        Some(val) => val.parse::<u8>().unwrap_or(1),
        None       => 5
    };

    println!("total = {}\n", board.perft(d, true));
}

fn uci() {
    println!("id name {}", ENGINE_NAME);
    println!("id author Alex Johnson");
    println!("uciok");
}

fn test_positions(path: &str, searcher: &mut Searcher, do_work: &mut FnMut(&mut Searcher)) {
    let file = match File::open(path) {
        Ok(file) => BufReader::new(file),
        Err(e)   => panic!("Test suite {} could not be read. {:?}", path, e)
    };

    let start = time::precise_time_s();

    for line in file.lines().take(10) {
        let fen = String::from("fen ") + &line.unwrap();
        println!("{}", fen);

        searcher.position(&mut fen.split_whitespace());
        do_work(searcher);
    }
    println!("Time taken = {} seconds", time::precise_time_s() - start);
}
