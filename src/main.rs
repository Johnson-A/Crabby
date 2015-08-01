#![feature(slice_patterns, convert, negate_unsigned, append, test, associated_consts, const_fn)]
extern crate test;
extern crate time;
extern crate rand;

use std::io;
use std::io::prelude::*;

use types::*;
mod types;
mod board;
mod util;
mod evaluation;
mod search;
use table::Table;
mod table;
mod magics;

const ENGINE_NAME: &'static str = "Prototype Chess Engine";

fn main() {
    unsafe {
        magics::init();
        table::init();
    }
    // let mut fen = "r5k1/1bpnqrpp/pp2p3/3p4/N1PPnb2/1P1B1N2/PBR1QPPP/3R2K1 w - - 0 1".split(' ').collect();
    // let pos = Board::new_fen(&mut fen);
    // let pos = Board::new_default();
    // pos.negamax_a_b(7, -1000000, 1000000, &mut Vec::new());
    // tests();
    let stdin = io::stdin();
    let mut pos = Board::new_default();
    let mut table = Table::empty(10000000);
    let mut depth = 8;

    for line in stdin.lock().lines() {
        let line = line.unwrap_or("".to_string());
        let mut words: Vec<&str> = line.trim().split(' ').collect();
        let first_word = words.remove(0);

        match first_word {
            "uci"        => uci(),
            "setoption"  => (),
            "isready"    => println!("readyok"),
            "ucinewgame" => depth = 8, // new game
            "position"   => pos = position(&mut words),
            "go"         => go(&pos, &mut depth, &mut table),
            "print"      => (),
            _            => (), // Ignore any other command
        }
    }
}

fn go(board: &Board, depth: &mut u8, table: &mut Table) {
    println!("Searching\n{}", board);

    let start = time::precise_time_s();
    let mut pos = 1;
    let mut calc_time = start;

    while pos <= *depth {
        let (score, _) = board.negamax_a_b(pos, -std::i32::MAX, std::i32::MAX, table);
        calc_time = time::precise_time_s() - start;

        let mut pv = Vec::new();
        table.pv(&mut board.clone(), &mut pv);

        println!("info depth {} score cp {} time {} pv {}",
            pos, score / 10, (calc_time * 1000.0) as u32,
            pv.iter().map(|mv| mv.to_str()).collect::<Vec<_>>().join(" "));
        pos += 1;
    }

    // let mut max = 0;
    // for entry in &table.entries {
    //     if entry.depth > max {
    //         max = entry.depth;
    //         println!("found entry {}", max);
    //     }
    // }
    // println!("occ {} of {}", table.count(), table.entries.len());
    table.set_ancient();

    let best = table.best_move(board.hash);
    println!("bestmove {}", best.unwrap().to_str());

    if calc_time < 3.0 { *depth += 1; }
    if calc_time > 20.0 && *depth > 6 { *depth -= 1; }
}

fn position(params: &mut Vec<&str>) -> Board {
    let mut pos = match params.remove(0) { // ["startpos", "fen"]
        "startpos" => Board::new_default(),
        _fen       => Board::from_fen(params) // remove the fen string while creating board
    };

    if params.len() > 0 { params.remove(0); } // Remove "moves" string if there are moves

    for mv_str in params {
        pos.make_str_move(mv_str);
    }
    pos
}

fn uci() {
    println!("id name {}", ENGINE_NAME);
    println!("id author Alex Johnson");
    println!("uciok");
}

#[bench]
fn bench(b: &mut test::Bencher) {
    // use rand::Rng;
    // unsafe { if { MAP[0] } == 0 { init(); } }
    //
    // let mut rng = rand::thread_rng();
    // let c: u64 = rng.gen::<u64>() & rng.gen::<u64>();
    let mut res = 0;
    b.iter(|| test::black_box({
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
        println!("{}", res)
}
