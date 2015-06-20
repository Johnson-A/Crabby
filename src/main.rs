#![feature(slice_patterns, convert, test, negate_unsigned)]
use std::io;
use std::io::prelude::*;
extern crate rand;
extern crate test;
use rand::distributions::{IndependentSample, Range};

use piece::*;
mod piece;
use game::*;
mod game;
use board::*;
mod board;
use util::*;
mod util;

static ENGINE_NAME: &'static str = "Prototype Chess Engine";
static START_POS: &'static str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR";

#[bench]
fn bench(b: &mut test::Bencher) {
	// let mut t: u64 = 0;
	let board = Board::new("nN6/5BP1/1PR1PPKp/n2bpPbp/3Q1p1P/p1RpP1pN/qBP1rpk1/3r4");
	b.iter(|| test::black_box({
		// for i in 0u64..100000000u64 {
		// 	t |= reverse(i);
		// 	// t |= bit_scan_forward(i);
		// }
		for _ in 0..100 {
			board.get_moves(Color::White);
		}
	}
	));
	// println!("{}", t);
}

fn tests() {
	for c in "pnbrqk PNBRQK123=".chars() {
		println!("{:?}", to_piece(c));
	}
	println!("");

	let mut board = Board::new(START_POS);
	println!("{}", board);
	let moves = board.get_moves(Color::White);
	println!("moves = {:?}", moves);

	board.make_str_move("e2e4");
	board.make_str_move("e7e5");
	println!("{}", board);

	let game = Game {
		board: board, to_move: Color::White, 
		w_castle: true, b_castle: true, w_time: 1, b_time: 1,
		move_num: 1, en_pessant: 9};

	println!("{}", game);
}

fn main() {
	// tests();

	let stdin = io::stdin();
	let mut pos = Board::new(START_POS);
	
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
			"go"         => go(&pos),
			"print"      => print(),
			_            => continue, // Ignore any other command
		}
	}
}

fn go(board: &Board) {
	let moves = board.get_moves(Color::White);
	let between = Range::new(0, moves.len());
	let mut rng = rand::thread_rng();

	let mv = &moves[between.ind_sample(&mut rng)];
	println!("{:?}", moves.iter().map(|x| move_to_str(x.from, x.to)).collect::<Vec<String>>());
	println!("{:?}", mv);
	println!("bestmove {}", move_to_str(mv.from, mv.to));
}

fn print() {

}

fn position(params: &mut Vec<&str>) -> Board {
	let cmd = params.remove(0);
	let mut pos = match cmd {
		"startpos" => Board::new(START_POS),
		fen => Board::new(fen)
	};
	if params.len() > 0 {params.remove(0);} // Remove "moves" string

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
