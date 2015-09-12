use std::io::prelude::*;
use std::io::{stdin, BufReader};
use std::fs::File;
use std::sync::{Arc, Mutex};
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
    let mut init_proc = Some(thread::spawn(|| init()));
    let stdin = stdin();
    let table_size = 50_000_000;
    let searcher = ArcMutex!(Searcher::new_start(table_size));
    let timer = ArcMutex!(Timer::new());

    for line in stdin.lock().lines() {
        let line = line.unwrap_or("".into());
        let mut params = line.split_whitespace();

        if let Some(first_word) = params.next() {
            match first_word {
                "uci"        => uci(),
                "setoption"  => (),
                "isready"    => println!("readyok"),
                "ucinewgame" => lock!(searcher).refresh(),
                "position"   => lock!(searcher).position(&mut params),
                "go"         => {
                    finish(&mut init_proc);
                    let searcher = searcher.clone();
                    *lock!(timer) = Timer::parse(Timer::new(), &mut params);
                    let timer = timer.clone();

                    thread::spawn(move || {
                        lock!(searcher).go(timer);
                    });
                },
                "perft"      => perft(&lock!(searcher).root, &mut params),
                "testperf" |
                "testmove" => {
                    finish(&mut init_proc);
                    match first_word {
                        "testperf" => positions("testing/positions/performance",
                                          &mut lock!(searcher), &mut |s, t| s.go(t)),
                        _ => positions("testing/positions/perftsuite.epd",
                                &mut lock!(searcher), &mut |s, _| println!("{}", s.root.perft(6, true)))
                    };
                },
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

pub fn positions(path: &str, searcher: &mut Searcher,
                    do_work: &mut FnMut(&mut Searcher, Arc<Mutex<Timer>>)) {
    let file = match File::open(path) {
        Ok(file) => BufReader::new(file),
        Err(e)   => panic!("Test suite {} could not be read. {:?}", path, e)
    };

    let start = time::precise_time_s();

    for line in file.lines().take(10) {
        let fen = String::from("fen ") + &line.unwrap();
        println!("{}", fen);

        let mut params = "wtime 100000 btime 100000 movestogo 1".split_whitespace();
        let timer = Timer::parse(Timer::new(), &mut params);
        searcher.position(&mut fen.split_whitespace());
        do_work(searcher, Arc::new(Mutex::new(timer)));
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
