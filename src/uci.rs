use std::io::prelude::*;
use std::io::{stdin, BufReader};
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use time;

use types::*;
use util::*;
use board::Board;
use magics;
use table;
use search::Searcher;
use timer::Timer;

const ENGINE_NAME: &'static str = "Crabby";

pub fn main_loop() {
    let init_proc = &mut Some(thread::spawn(init));
    let should_stop = Arc::new(AtomicBool::new(false)); // Should not be exposed
    let timer = Timer::default(should_stop.clone());
    let searcher = Arc::new(Mutex::new(Searcher::new(EngineSettings::default(), timer)));

    let stdin = stdin();
    for line in stdin.lock().lines() {
        let line = line.unwrap_or("".into());
        let mut params = line.split_whitespace();

        if let Some(first_word) = params.next() {
            match first_word {
                "uci"        => uci(),
                "isready"    => println!("readyok"),
                "setoption"  => lock!(searcher).update_settings(&mut params),
                "ucinewgame" => lock!(searcher).reset(),
                "position"   => lock!(searcher).position(&mut params),
                "stop"       => should_stop.store(true, Ordering::Relaxed),
                "quit"       => return,
                "print"      => (),
                "go" | "perft" | "test" => {
                    finish(init_proc);

                    match first_word {
                        "go" => {
                            lock!(searcher).timer.replace(&mut params);

                            let searcher = searcher.clone();
                            thread::spawn(move || {
                                lock!(searcher).go();
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

#[derive(Copy, Clone)]
pub struct EngineSettings {
    pub table_size: usize
}

impl Default for EngineSettings {
    fn default() -> EngineSettings {
        EngineSettings {
            table_size: 10_000_000
        }
    }
}

pub fn run(searcher: &mut Searcher, test: Option<&str>) {
    match test {
        Some("perf") => positions("testing/positions/performance",
                searcher, &mut |s| s.go()),
        Some("move") => positions("testing/positions/perftsuite.epd",
                searcher, &mut |s| println!("{}", s.root.perft(6, true))),
        _ => println!("Valid options are `perf` or `move`")
    };
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
    searcher.timer.replace(&mut "wtime 10000 btime 10000 movestogo 1".split_whitespace());

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
    println!("option name Hash type spin min 2 default 128");
    println!("uciok");
}
