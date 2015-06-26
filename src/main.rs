#![feature(slice_patterns, convert, negate_unsigned, associated_consts)]
#[macro_use]
extern crate lazy_static;

use std::io;
use std::io::prelude::*;

use types::*;
mod types;
use board::*;
mod board;
mod util;

static ENGINE_NAME: &'static str = "Prototype Chess Engine";

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

fn main() {
    tests();

    let stdin = io::stdin();
    let mut pos = Board::new_default();

    for line in stdin.lock().lines() {
        let line = line.unwrap_or("".to_string());
        let mut words: Vec<&str> = line.trim().split(' ').collect();
        let first_word = words.remove(0);

        match first_word {
            "uci"        => uci(),
            "setoption"  => (),
            "isready"    => println!("readyok"),
            "ucinewgame" => (), // new game
            "position"   => pos = position(&mut words),
            "go"         => go(&mut pos),
            "print"      => (),
            _            => (), // Ignore any other command
        }
    }
}

fn go(board: &mut Board) {
    let (score, mv) = board.negamax_a_b(6 , -10000.0, 10000.0);
    println!("info depth 1 currmove {} multipv 1 score cp {}", mv.to_str(), (score*100.0) as i32);
    println!("bestmove {}", mv.to_str());
}

fn position(params: &mut Vec<&str>) -> Board {
    let mut pos = match params[0] {
        "startpos" => {
            params.remove(0);
            Board::new_default()
        },
        _ => Board::new_fen(params) // remove the fen string while creating board
    };

    if params.len() > 0 { params.remove(0); } // Remove "moves" string if there are moves

    for mv_str in params {
        pos.make_str_move(mv_str);
    }

    println!("{}", pos);
    pos
}

fn uci() {
    println!("id name {}", ENGINE_NAME);
    println!("id author Alex Johnson");
    println!("uciok")
}
