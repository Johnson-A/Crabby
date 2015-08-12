//! This is the Crabby chess engine
#![feature(slice_patterns, convert, negate_unsigned, append, test, associated_consts, const_fn)]
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

use table::Table;
use types::*;
use search::Searcher;

const ENGINE_NAME: &'static str = "Crabby";

pub fn main() {
    unsafe {
        magics::init();
        table::init();
    }

    let stdin = stdin();
    let mut pos = Board::start_position();
    let mut table = Table::empty(10000000);
    let mut depth = 8;

    for line in stdin.lock().lines() {
        let line = line.unwrap_or("".into());
        let mut words: Vec<&str> = line.trim().split(' ').collect();
        let first_word = words.remove(0);

        match first_word {
            "uci"        => uci(),
            "setoption"  => (),
            "isready"    => println!("readyok"),
            "ucinewgame" => depth = 8,
            "position"   => pos = position(&mut words),
            "go"         => go(&pos, &mut depth, &mut table),
            "ponder"     => go(&pos, &mut 255, &mut table), // TODO: implement stop signal
            "moves"      => make_moves(&mut pos, &mut words),
            "perft"      => perft(&pos, &mut words),
            "test"       => test_positions("test_positions/positions", &mut table),
            "print"      => (),
            _            => (), // Ignore any other command
        }
    }
}

pub fn perft(board: &Board, params: &mut Vec<&str>) {
    let d = if params.len() >= 1 {
        params.remove(0).parse::<u8>().unwrap_or(5)
    } else { 5 };

    println!("total = {}\n", board.perft(d, true));
}

pub fn go(board: &Board, depth: &mut u8, table: &mut Table) {
    println!("Searching\n{}", board);
    for mv in &board.get_moves() {
        println!("({}, {})", board.see_move(mv), mv)
    }
    let calc_time = {
        let mut search = Searcher::new(*depth, board, table);
        search.id()
    };
    println!("occ {} of {}", table.count(), table.entries.len());
    table.set_ancient();

    let best = table.best_move(board.hash);
    println!("bestmove {}", best.unwrap());

    if calc_time < 1.0 { *depth += 1; }
    if calc_time > 20.0 && *depth > 6 { *depth -= 1; }
}

fn position(params: &mut Vec<&str>) -> Board {
    let mut pos = match params.remove(0) { // ["startpos", "fen"]
        "startpos" => Board::start_position(),
        _fen       => Board::from_fen(params) // removes the fen string while creating board
    };

    if params.len() > 0 { params.remove(0); } // Remove "moves" string if there are moves
    make_moves(&mut pos, params);

    pos
}

fn uci() {
    println!("id name {}", ENGINE_NAME);
    println!("id author Alex Johnson");
    println!("uciok");
}

fn test_positions(path: &str, table: &mut Table) {
    let file = match File::open(path) {
        Ok(file) => BufReader::new(file),
        Err(e)  => panic!("Test suite {} could not be read. {:?}", path, e)
    };

    let start = time::precise_time_s();
    // 443 seconds -> (150) seconds
    for line in file.lines().take(10) {
        let fen = line.unwrap();
        let board = Board::from_fen(&mut fen.split(' ').collect());
        println!("{}", fen);
        go(&board, &mut 8, table);
    }
    println!("Time taken = {} seconds", time::precise_time_s() - start);
}

#[bench]
fn bench(b: &mut test::Bencher) {
    unsafe {
        if magics::KING_MAP[0] == 0 {
            magics::init();
            table::init();
        }
    }
    // use rand::Rng;
    // unsafe { if { MAP[0] } == 0 { init(); } }
    //
    // let mut rng = rand::thread_rng();
    // let c: u64 = rng.gen::<u64>() & rng.gen::<u64>();
    let board = Board::start_position();
    b.iter(|| test::black_box({
        board.get_moves();
        board.get_moves();
        board.get_moves();
        board.get_moves();
        board.get_moves();

        board.get_moves();
        board.get_moves();
        board.get_moves();
        board.get_moves();
        board.get_moves();

        // unsafe {
        // res |= BISHOP_MAP[0].att(c);
        // res |= BISHOP_MAP[0].att(c);
        // res |= BISHOP_MAP[10].att(c);
        // res |= BISHOP_MAP[20].att(c);
        // res |= BISHOP_MAP[10].att(c);
        // res |= BISHOP_MAP[20].att(c);
        // res |= BISHOP_MAP[30].att(c);
        // res |= BISHOP_MAP[1].att(c);
        // res |= BISHOP_MAP[40].att(c);
        // res |= BISHOP_MAP[20].att(c);
        //
        // res |= ROOK_MAP[0].att(c);
        // res |= ROOK_MAP[0].att(c);
        // res |= ROOK_MAP[10].att(c);
        // res |= ROOK_MAP[20].att(c);
        // res |= ROOK_MAP[10].att(c);
        // res |= ROOK_MAP[20].att(c);
        // res |= ROOK_MAP[30].att(c);
        // res |= ROOK_MAP[1].att(c);
        // res |= ROOK_MAP[40].att(c);
        // res |= ROOK_MAP[20].att(c);
        // }
        }));
        // println!("{}", t);
}
