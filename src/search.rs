use std::i32;
use std::cmp::{min, max};
use timer::Timer;
use types::*;
use _move::*;
use board::Board;
use table::*;
use uci::EngineSettings;
use util::parse;

pub const INFINITY: i32 = i32::MAX;
pub const VALUE_MATE: i32 = INFINITY / 2;

#[derive(PartialEq, Eq)]
pub enum NT {
    Root, PV, NonPV
}

pub struct Searcher {
    pub root: Board,
    pub timer: Timer,
    pub settings: EngineSettings,
    table: Table,
    killers: Vec<Killer>,
    rep: Vec<Hash>,
    ply: usize,
    node_count: usize,
    irreversible: usize
}

impl Searcher {
    /// Create a new searcher from the start position
    pub fn new(settings: EngineSettings, timer: Timer) -> Self {
        let start = Board::start_position();

        Searcher {
            root: start,
            timer: timer,
            settings: settings,
            table: Table::empty(settings.table_size),
            killers: vec![Killer::EMPTY],
            rep: vec![start.hash],
            ply: 0,
            node_count: 0,
            irreversible: 0
        }
    }

    pub fn update_settings(&mut self, params: &mut Params) {
        while let Some(option) = params.next() {
            if option == "name" {
                match &*params.next().unwrap().to_lowercase() {
                    "hash" => {
                        let size_mb = parse(params.nth(1));
                        self.table = Table::empty_mb(size_mb);
                        self.settings.table_size = self.table.num_elems();
                    },
                    _ => ()
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.table.units = vec![]; // Explicitly drop the previous table
        *self = Searcher::new(self.settings, Timer::default(self.timer.should_stop.clone()));
    }

    pub fn extend(&mut self) {
        self.killers.push(Killer::EMPTY);
        self.rep.push(Hash { val: 0 });
    }

    pub fn position(&mut self, params: &mut Params) {
        self.root = match params.next().expect("[startpos, fen]") {
            "startpos" => Board::start_position(),
            _fen       => Board::from_fen(params)
        };

        self.node_count = 0;
        self.rep = vec![self.root.hash];
        self.killers = vec![];

        // Remove half move, full move, and other words until there are moves
        for mv_str in params.skip_while(|&val| val != "moves").skip(1) {
            let mv = self.root.move_from_str(mv_str);
            if self.root.is_irreversible(mv) {
                self.irreversible = self.root.ply + 1;
            }
            self.root.make_move(mv);
            self.rep.push(self.root.hash);
        }
    }

    pub fn go(&mut self) {
        assert!(self.ply == 0, "Search must start at ply 0");
        println!("Searching\n{}", self.root);
        self.timer.start(self.root.to_move);
        let mut depth = 1;

        while self.timer.should_search(depth) {
            self.extend();
            let root = self.root; // Needed due to lexical borrowing (which will be resolved)
            let score = self.search(&root, depth as u8, -INFINITY, INFINITY, NT::Root);

            self.timer.toc(self.node_count);
            let pv = self.table.pv(&self.root);
            let pv_str: Vec<String> = pv.iter().map(|mv| format!("{}", mv)).collect();

            println!("info depth {} score cp {} time {} nodes {} pv {}",
                depth, score / 10, (self.timer.elapsed() * 1000.0) as u32,
                self.node_count, pv_str.join(" "));

            depth += 1;
        }

        println!("occ {} of {}", self.table.set_ancient(), self.table.num_elems());

        let best = self.table.best_move(self.root.hash);
        println!("bestmove {}", best.unwrap_or(Move::NULL));
    }

    pub fn search(&mut self, board: &Board, depth: u8, mut alpha: i32, beta: i32, nt: NT) -> i32 {
        self.node_count += 1;
        if board.player_in_check(board.prev_move()) { return INFINITY }

        let (table_score, mut best_move) = self.table.probe(board.hash, depth, alpha, beta);

        if let Some(s) = table_score {
            return s
        }

        if depth == 0 {
            let score = self.q_search(&board, 8, alpha, beta);
            self.table.record(board, score, Move::NULL, depth, Bound::Exact);
            return score
        }

        let mut best_value = -INFINITY;
        let is_pv = nt == NT::Root || nt == NT::PV;

        if    !is_pv
           && depth >= 2
           && !board.is_in_check()
        {
            let eval = board.evaluate();
            let r = 3 + depth as i32 / 4 + min(max(eval - beta, 0) / p_val(PAWN) as i32, 3);
            let mut new_board = *board;
            new_board.do_null_move();

            let d = if r as u8 >= depth { 0 } else { depth - r as u8 };
            self.ply += 1;
            let s = -self.search(&new_board, d, -beta, -beta+1, NT::NonPV);
            self.ply -= 1;

            if s >= beta {
                if s >= VALUE_MATE - 1000 { return beta }

                if depth < 14 { return s }
                let v = self.search(&board, d, beta - 1, beta, NT::NonPV);
                if v >= beta { return s }
            }
        }

        let moves = board.sort_with(board.get_moves(), best_move, &self.killers[self.ply]);

        let mut moves_searched = 0;

        for (_, mv) in moves {
            let mut new_board = *board;
            new_board.make_move(mv);

            self.ply += 1;

            let score = if self.check_repitition(new_board.hash) {
                0
            } else if moves_searched == 0 {
                -self.search(&new_board, depth - 1, -beta, -alpha, NT::PV)
            } else {
                let mut s = alpha + 1;

                if    depth >= 3
                   && !mv.is_capture()
                   && mv.promotion() == 0
                   && mv != self.killers[self.ply].0
                   && mv != self.killers[self.ply].1
                   && !new_board.is_in_check()
                {
                    let d = depth - depth / 5 - 2;
                    s = -self.search(&new_board, d, -(alpha+1), -alpha, NT::NonPV);
                }

                if s > alpha {
                    s = -self.search(&new_board, depth - 1, -(alpha+1), -alpha, NT::NonPV);
                    if s > alpha && s < beta {
                        s = -self.search(&new_board, depth - 1, -beta, -alpha, NT::NonPV);
                    }
                }
                s
            };
            self.ply -= 1;

            if score != -INFINITY { moves_searched += 1 } else { continue }

            if score > best_value {
                best_move = mv;
                best_value = score;
                if score >= beta {
                    if !mv.is_capture() { self.killers[self.ply].substitute(mv) }
                    self.table.record(board, score, mv, depth, Bound::Lower);
                    return score
                }
                alpha = max(alpha, score);
            }
        }

        if moves_searched == 0 {
            if board.is_in_check() {
                return -VALUE_MATE + self.ply as i32
            } else {
                return 0
            }
        }

        self.table.record(board, best_value, best_move, depth, Bound::Upper);
        best_value
    }

    // TODO: update irreversible, full three move and fifty move repition
    pub fn check_repitition(&mut self, hash: Hash) -> bool {
        let mut pos_ply = self.ply + self.root.ply;
        self.rep[pos_ply] = hash;

        let last_index = max(2, self.irreversible);

        while pos_ply >= last_index {
            pos_ply -= 2;
            if self.rep[pos_ply] == hash {
                return true
            }
        }
        false
    }

    pub fn q_search(&mut self, board: &Board, depth: u8, mut alpha: i32, beta: i32) -> i32 {
        self.node_count += 1;
        if board.player_in_check(board.prev_move()) { return INFINITY }
        let stand_pat = board.evaluate();
        if depth == 0 || stand_pat >= beta { return stand_pat }
        if stand_pat > alpha { alpha = stand_pat }

        for (_, mv) in board.qsort(&board.get_moves()) {
            let mut new_board = *board;
            new_board.make_move(mv);
            let score = -self.q_search(&new_board, depth - 1, -beta, -alpha);

            if score > alpha {
                if score >= beta { return score }
                alpha = score;
            }
        }
        alpha
    }
}
