#![feature(slice_patterns, convert, negate_unsigned, append, test)]
extern crate time;
extern crate rand;
extern crate test;

use std::io;
use std::io::prelude::*;

use types::*;
mod types;
mod board;
use util::*;
mod util;
mod evaluation;

#[bench]
fn bench(b: &mut test::Bencher) {
    use rand::Rng;
    if unsafe { MAP[0] } == 0 { init(); }

    let mut rng = rand::thread_rng();
    let c: u64 = rng.gen::<u64>() & rng.gen::<u64>();
    let mut res = 0;
    b.iter(|| test::black_box({
        unsafe {
        res |= BISHOP_MAP[0].att(c);
        res |= BISHOP_MAP[0].att(c);
        res |= BISHOP_MAP[10].att(c);
        res |= BISHOP_MAP[20].att(c);
        res |= BISHOP_MAP[10].att(c);
        res |= BISHOP_MAP[20].att(c);
        res |= BISHOP_MAP[30].att(c);
        res |= BISHOP_MAP[1].att(c);
        res |= BISHOP_MAP[40].att(c);
        res |= BISHOP_MAP[20].att(c);

        res |= ROOK_MAP[0].att(c);
        res |= ROOK_MAP[0].att(c);
        res |= ROOK_MAP[10].att(c);
        res |= ROOK_MAP[20].att(c);
        res |= ROOK_MAP[10].att(c);
        res |= ROOK_MAP[20].att(c);
        res |= ROOK_MAP[30].att(c);
        res |= ROOK_MAP[1].att(c);
        res |= ROOK_MAP[40].att(c);
        res |= ROOK_MAP[20].att(c);
        }
        }));
        println!("{}", res)
}


static ENGINE_NAME: &'static str = "Prototype Chess Engine";

fn main() {
    init();
    // let mut fen = "r5k1/1bpnqrpp/pp2p3/3p4/N1PPnb2/1P1B1N2/PBR1QPPP/3R2K1 w - - 0 1".split(' ').collect();
    // let pos = Board::new_fen(&mut fen);
    // let pos = Board::new_default();
    // pos.negamax_a_b(7, -1000000, 1000000, &mut Vec::new());
    // tests();
    let stdin = io::stdin();
    let mut pos = Board::new_default();
    let mut depth = 6;

    for line in stdin.lock().lines() {
        let line = line.unwrap_or("".to_string());
        let mut words: Vec<&str> = line.trim().split(' ').collect();
        let first_word = words.remove(0);

        match first_word {
            "uci"        => uci(),
            "setoption"  => (),
            "isready"    => println!("readyok"),
            "ucinewgame" => depth = 6, // new game
            "position"   => pos = position(&mut words),
            "go"         => go(&pos, &mut depth),
            "print"      => (),
            _            => (), // Ignore any other command
        }
    }
}

fn go(board: &Board, depth: &mut u32) {
    println!("Searching\n{}", board);

    let start = time::precise_time_s();
    let mut pv = Vec::new();
    let (score, _) = board.negamax_a_b(*depth, -std::i32::MAX, std::i32::MAX, &mut pv);
    let calc_time = time::precise_time_s() - start;

    println!("info depth {} score cp {} time {} pv {}",
        depth, score / 10, (calc_time * 1000.0) as u32,
        pv.iter().map(|mv| mv.to_str()).collect::<Vec<_>>().connect(" "));
    println!("bestmove {}", pv[0].to_str());

    if calc_time < 0.5 { *depth += 1; }
    if (calc_time > 20.0) & (*depth > 6) { *depth -= 1; }
}

fn position(params: &mut Vec<&str>) -> Board {
    let mut pos = match params.remove(0) { // ["startpos", "fen"]
        "startpos" => Board::new_default(),
        _fen 	   => Board::new_fen(params) // remove the fen string while creating board
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

fn tests() {
    for c in "pnbrqk PNBRQK123=".chars() {
        println!("{:?}", to_piece(c));
    }
    println!("");

    let mut board = Board::new_default();
    println!("Start eval {}", board.evaluate());
    println!("{}", board);
    let moves = board.get_moves();
    println!("moves = {:?}", moves.iter().map(|mv| mv.to_str()).collect::<Vec<String>>());

    board.make_str_move("e2e4");
    println!("e2e4 eval {}", board.evaluate());
    board.make_str_move("d7d5");
    println!("e7e5 eval {}", board.evaluate());
    board.make_str_move("e4e5");
    board.make_str_move("f7f5");
    println!("moves = {:?}", board.get_moves().iter().map(|mv| mv.to_str()).collect::<Vec<String>>());
    board.make_str_move("e5f6");
    board.get_moves();

    println!("{}", board);
}
