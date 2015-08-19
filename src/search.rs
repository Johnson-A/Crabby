use std::i32;
use std::cmp::{min, max};
use time;
use types::*;
use table::*;

pub const INFINITY: i32 = i32::MAX;
pub const VALUE_MATE: i32 = 1000000000;

#[derive(PartialEq, Eq)]
pub enum NT {
    Root, PV, NonPV
}

pub struct Searcher<'a> {
    pub board: &'a Board,
    pub table: &'a mut Table,
    pub rep: Vec<Hash>,
    pub killers: Vec<Killer>,
    pub max_ply: u8,
    pub ply: usize,
    pub node_count: usize,
    pub q_count: usize
}

impl<'a> Searcher<'a> {
    pub fn new(max: u8, board: &'a Board, table: &'a mut Table) -> Searcher<'a> {
        Searcher {
            board: board,
            table: table,
            rep: vec![Hash { val: 0 }; max as usize],
            killers: vec![Killer(Move::NULL, Move::NULL); max as usize],
            max_ply: max,
            ply: 0,
            node_count: 0,
            q_count: 0
        }
    }

    /// Search up to max_ply and return an estimate for a good search depth next move
    pub fn id(&mut self) -> u8 {
        // TODO: keep track of all ply, self.ply = self.move_num
        let start = time::precise_time_s();
        let mut calc_time = 0.0;
        let mut depth = 1;

        while (calc_time < 5.0 && depth <= self.max_ply) || self.max_ply == 255 {
            let score = self.search(self.board, depth, -INFINITY, INFINITY, NT::Root);

            calc_time = time::precise_time_s() - start;
            let pv = self.table.pv(self.board);

            println!("info depth {} score cp {} time {} nodes {} pv {}",
                depth, score / 10, (calc_time * 1000.0) as u32, self.node_count + self.q_count,
                pv.iter().map(|mv| mv.to_string()).collect::<Vec<_>>().join(" "));

            depth += 1;
        }

        println!("occ {} of {}", self.table.count(), self.table.entries.len());
        self.table.set_ancient();

        let best = self.table.best_move(self.board.hash);
        println!("bestmove {}", best.unwrap());
        depth
    }

    // TODO: wrap self.ply += 1 /* search */ self.ply -= 1
    pub fn search(&mut self, board: &Board, depth: u8, mut alpha: i32, beta: i32,
                             nt: NT) -> i32 {
        self.node_count += 1;
        if board.player_in_check(board.prev_move()) { return INFINITY }

        let is_pv = nt == NT::Root || nt == NT::PV;

        let (table_score, mut best_move) = self.table.probe(board.hash, depth, alpha, beta);

        if let Some(s) = table_score {
            return s
        }

        if depth == 0 {
            let score = self.q_search(&board, 8, alpha, beta);
            self.table.record(board, score, Move::NULL, depth, NodeBound::Exact);
            return score
        }

        let eval = board.evaluate();

        if    !is_pv
           && depth >= 2
           && eval >= beta
           && !board.is_in_check()
        {
            // Testing value from stockfish
            let r = (823 + 67 * (depth as i32)) / 256 + min((eval - beta) / p_val(PAWN) as i32, 3);

            let mut new_board = *board;
            new_board.do_null_move();

            let d = if r as u8 >= depth { 0 } else { depth - r as u8 };
            self.ply += 1;
            let s = -self.search(&new_board, d, -beta, 1-beta, NT::NonPV);
            self.ply -= 1;

            if s >= beta {
                if s >= VALUE_MATE - 1000 { return beta }

                if depth < 12 { return s }
                self.ply += 1;
                let v = self.search(&board, d, beta - 1, beta, NT::NonPV);
                self.ply -= 1;
                if v >= beta { return s }
            }
        }

        let moves = board.sort_with(&mut board.get_moves(), best_move,
                                    &self.killers[self.ply]);

        let mut moves_searched = 0;

        for (_, mv) in moves {
            let mut new_board = *board;
            new_board.make_move(mv);

            self.rep[self.ply] = new_board.hash;

            let mut rep_search = self.ply;
            let mut is_rep = false;

            while rep_search >= 2 {
                rep_search -= 2;
                if self.rep[rep_search] == new_board.hash {
                    is_rep = true;
                    break
                }
            }

            self.ply += 1;
            let score = match is_rep {
                true => 0,
                false if moves_searched == 0 =>
                    -self.search(&new_board, depth - 1, -beta, -alpha, NT::PV), // TODO: Null move in PVS?

                _ => {
                    // moves_searched > 0
                    let lmr =  depth >= 3
                            && moves_searched >= 2
                            && !mv.is_capture()
                            && mv.promotion() == 0
                            && mv != self.killers[self.ply].0
                            && mv != self.killers[self.ply].1
                            && !new_board.is_in_check();

                    let new_depth = depth - if lmr { 2 } else { 1 };

                    let s = -self.search(&new_board, new_depth, -(alpha+1), -alpha, NT::NonPV);

                    if s > alpha { // && s < beta - fail hard
                        -self.search(&new_board, new_depth, -beta, -s, NT::NonPV)
                    } else {
                        s
                    }
                }
            };
            self.ply -= 1;
            // table >= depth

            if score != -INFINITY { moves_searched += 1 } else { continue }

            if score >= beta {
                if !mv.is_capture() { self.killers[self.ply].substitute(mv) }
                self.table.record(board, score, mv, depth, NodeBound::Beta);
                return score
            }
            if score > alpha {
                best_move = mv;
                alpha = score;
            }
        }

        if moves_searched == 0 {
            if board.is_in_check() {
                return -VALUE_MATE + self.ply as i32
            } else {
                return 0
            }
        }

        self.table.record(board, alpha, best_move, depth, NodeBound::Alpha);
        alpha
    }

    pub fn q_search(&mut self, board: &Board, depth: u8, mut alpha: i32, beta: i32) -> i32 {
        self.q_count += 1;
        if board.player_in_check(board.prev_move()) { return INFINITY }
        // TODO: remove depth so all takes are searched
        // TODO: When no legal moves possible, return draw to avoid stalemate
        // TODO: Three move repition
        let stand_pat = board.evaluate();
        if depth == 0 || stand_pat >= beta { return stand_pat }
        if stand_pat > alpha { alpha = stand_pat }

        let mut captures = board.get_moves();
        captures.retain(|mv| mv.is_capture());

        for (_, mv) in board.sort(&captures) {
            let mut new_board = *board;
            new_board.make_move(mv);
            let score = -self.q_search(&new_board, depth - 1, -beta, -alpha);

            if score >= beta { return score }
            if score > alpha { alpha = score; }
        }
        alpha
    }
}
