#![feature(slice_patterns, convert, test, negate_unsigned)]
#[macro_use]
extern crate lazy_static;
extern crate threadpool;

use std::io;
use std::io::prelude::*;
extern crate test;

use piece::*;
mod piece;
use board::*;
mod board;
mod util;

static ENGINE_NAME: &'static str = "Prototype Chess Engine";
static START_POS: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

#[bench]
fn bench(b: &mut test::Bencher) {
    let mut board0 = Board::new(START_POS);
    let mut board1 = Board::new("nN6/5BP1/1PR1PPKp/n2bpPbp/3Q1p1P/p1RpP1pN/qBP1rpk1/3r4");
    let mut board2 = Board::new("6k1/8/K1b3n1/1P1PR2p/PP2Br2/3Q4/8/R3r1N1");
    let mut board3 = Board::new("8/1Pp1N1R1/1k2p3/1bnp2b1/nP2P1K1/4P3/Q1R2p1P/5N2");
    let mut board4 = Board::new("8/1RBp1pq1/R3PPP1/PrP1pPn1/P2k1rB1/b1p1Nnp1/2pppP2/1QNb3K");

    b.iter(|| test::black_box({
        for _ in 0..2 {
            board0.best_move();
            board1.best_move();
            board2.best_move();
            board3.best_move();
            board4.best_move();
        }
    }));
}

fn tests() {
    for c in "pnbrqk PNBRQK123=".chars() {
        println!("{:?}", to_piece(c));
    }
    println!("");

    let mut board = Board::new(START_POS);
    println!("Start eval {}", board.evaluate());
    println!("{}", board);
    let moves = board.get_moves();
    println!("moves = {:?}", moves.iter().map(move_to_str).collect::<Vec<String>>());

    board.make_str_move("e2e4");
    println!("e2e4 eval {}", board.evaluate());
    board.make_str_move("e7e5");
    println!("e7e5 eval {}", board.evaluate());

    println!("{}", board);
}

fn main() {
    tests();

    let mut pos = Board::new(START_POS);
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line.unwrap_or("".to_string());
        let mut words: Vec<&str> = line.trim().split(' ').collect();
        let first_word = words.remove(0);

        match first_word {
            "uci"        => uci(),
            "setoption"  => set_option(&words),
            "isready"    => println!("readyok"),
            "ucinewgame" => continue, // new game
            "position"   => pos = position(&mut words),
            "go"         => go(&mut pos),
            "print"      => print(),
            _            => continue, // Ignore any other command
        }
    }
}

fn go(board: &mut Board) {
    let mv = board.best_move();
    // println!("negamax = {}", board.negamax(4));
    println!("bestmove {}", move_to_str(&mv));
}

fn print() {

}

fn position(params: &mut Vec<&str>) -> Board {
    let cmd = params.remove(0);
    let mut pos = match cmd {
        "startpos" => Board::new(START_POS),
        fen => Board::new(fen)
    };
    if params.len() > 0 { params.remove(0); } // Remove "moves" string

    for mv_str in params {
        pos.make_str_move(mv_str);
    }

    println!("{}", pos);
    pos
}

fn set_option(params: &Vec<&str>) {

}

fn uci() {
    println!("id name {}", ENGINE_NAME);
    println!("id author Alex Johnson");
    println!("uciok")
}
