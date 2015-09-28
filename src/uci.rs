use std::io::prelude::*;
use std::io::{stdin, BufReader};
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use time;

use types::*;
use util::*;
use magics;
use table;
use search::Searcher;
use timer::Timer;

const ENGINE_NAME: &'static str = "Crabby";

pub fn main_loop() {
    let init_proc = &mut Some(thread::spawn(|| init()));
    let stdin = stdin();
    let table_size = 50_000_000;
    let should_stop = Arc::new(AtomicBool::new(false));
    let searcher = ArcMutex!(Searcher::new_start(table_size, should_stop.clone()));

    for line in stdin.lock().lines() {
        let line = line.unwrap_or("".into());
        let mut params = line.split_whitespace();

        if let Some(first_word) = params.next() {
            match first_word {
                "uci"        => uci(),
                "setoption"  => (),
                "isready"    => println!("readyok"),
                "ucinewgame" => lock!(searcher).refresh(table_size),
                "position"   => lock!(searcher).position(&mut params),
                "stop"       => should_stop.store(true, Ordering::Relaxed),
                "quit"       => return,
                "print"      => (),
                "go" | "perft" | "test" => {
                    finish(init_proc);

                    match first_word {
                        "go" => {
                            let searcher = searcher.clone();
                            let timer = Timer::new(&mut params);

                            thread::spawn(move || {
                                lock!(searcher).go(timer);
                            });
                        },
                        "perft" => perft(&lock!(searcher).root, &mut params),
                        _test   => run(&mut lock!(searcher), params.next())
                    }
                }
                _ => println!("Unknown command: {}", first_word)
            }
        }
    }
}

pub fn run(searcher: &mut Searcher, test: Option<&str>) {
    match test {
        Some("perf") => positions("testing/positions/performance",
                searcher, &mut |s, t| s.go(t)),
        Some("move") => positions("testing/positions/perftsuite.epd",
                searcher, &mut |s, _| println!("{}", s.root.perft(6, true))),
        _ => println!("Valid options are `perf` or `move`")
    };
}

pub fn perft(board: &Board, params: &mut Params) {
    let d = parse_or(params.next(), 5);

    println!("total = {}\n", board.perft(d, true));
}

pub fn positions(path: &str, searcher: &mut Searcher,
                    do_work: &mut FnMut(&mut Searcher, Timer)) {
    let file = match File::open(path) {
        Ok(file) => BufReader::new(file),
        Err(e)   => panic!("Test suite {} could not be read. {:?}", path, e)
    };

    let start = time::precise_time_s();

    for line in file.lines().take(10) {
        let fen = String::from("fen ") + &line.unwrap();
        println!("{}", fen);

        let mut params = "wtime 100000 btime 100000 movestogo 1".split_whitespace();
        searcher.position(&mut fen.split_whitespace());
        do_work(searcher, Timer::new(&mut params));
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
