use std::io::prelude::*;
use std::io::{stdin, BufReader};
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::thread;
use time;

use types::*;
use util::parse_or;
use magics;
use table;
use search::Searcher;
use timer::Timer;

const ENGINE_NAME: &'static str = "Crabby";

pub fn main_loop() {
    let mut init_proc = Some(thread::spawn(|| init()));
    let stdin = stdin();
    let searcher = Arc::new(Mutex::new(Searcher::new_start()));
    let timer = Arc::new(Mutex::new(Timer::new()));

    for line in stdin.lock().lines() {
        let line = line.unwrap_or("".into());
        let mut params = line.split_whitespace();

        if let Some(first_word) = params.next() {
            match first_word {
                "uci"        => uci(),
                "setoption"  => (),
                "isready"    => println!("readyok"),
                "ucinewgame" => *lock!(searcher) = Searcher::new_start(),
                "position"   => lock!(searcher).position(&mut params),
                "go"         => {
                    match init_proc.take() {
                        Some(f) => f.join().is_ok(),
                        None => false
                    };
                    let searcher = searcher.clone();
                    *lock!(timer) = Timer::parse(Timer::new(), &mut params);
                    let timer = timer.clone();

                    thread::spawn(move || {
                        lock!(searcher).go(timer);
                    });
                },
                "perft"      => perft(&lock!(searcher).root, &mut params),
                // "testperf"   => positions("testing/positions/performance", &mut searcher, &mut |s| s.id()),
                "testmove"   => positions("testing/positions/perftsuite.epd", &mut lock!(searcher),
                                                &mut |s| println!("{}", s.root.perft(6, true))),
                "print"      => (),
                "stop"       => lock!(timer).stop = true,
                "quit"       => return,
                _            => println!("Unknown command: {}", first_word)
            }
        }
    }
}

pub fn perft(board: &Board, params: &mut Params) {
    let d = parse_or(params.next(), 5);

    println!("total = {}\n", board.perft(d, true));
}

pub fn positions(path: &str, searcher: &mut Searcher, do_work: &mut FnMut(&mut Searcher)) {
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

pub fn init() {
    unsafe {
        magics::init();
        table::init();
    }
}

pub fn uci() {
    println!("id name {}", ENGINE_NAME);
    println!("id author Alex Johnson");
    println!("uciok");
}
