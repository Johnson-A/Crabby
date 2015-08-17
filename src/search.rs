use std::i32;
use time;
use types::*;
use table::*;

pub struct Searcher<'a> {
    pub board: &'a Board,
    pub table: &'a mut Table,
    pub killers: Vec<Killer>,
    pub max_ply: u8,
    pub cur_depth: u8,
    pub node_count: usize
}

impl<'a> Searcher<'a> {
    pub fn new(max: u8, board: &'a Board, table: &'a mut Table) -> Searcher<'a> {
        Searcher {
            board: board,
            table: table,
            killers: vec![Killer(Move::NULL, Move::NULL); max as usize],
            max_ply: max,
            cur_depth: 1,
            node_count: 0
        }
    }

    pub fn id(&mut self) -> f64 {
        let start = time::precise_time_s();
        let mut calc_time = start;

        while !self.is_finished() {
            let depth = self.cur_depth;

            let (score, _) = self.negamax_a_b(self.board, depth, -i32::MAX, i32::MAX, true);
            // let (score, _) = self.pv_search(self.board, depth, -i32::MAX, i32::MAX);

            calc_time = time::precise_time_s() - start;
            let pv = self.table.pv(self.board);

            println!("info depth {} score cp {} time {} pv {}",
                depth, score / 10, (calc_time * 1000.0) as u32,
                pv.iter().map(|mv| mv.to_string()).collect::<Vec<_>>().join(" "));

            self.cur_depth += 1;
        }
        calc_time
    }

    pub fn is_finished(&self) -> bool { self.cur_depth > self.max_ply }

    // TODO: Fail soft, retain the pv
    pub fn negamax_a_b(&mut self, board: &Board, depth: u8, mut alpha: i32, beta: i32, allow_null: bool) -> (i32, bool) {
        if board.player_in_check(board.prev_move()) { return (0, false) }

        let (table_score, mut best_move) = self.table.probe(board.hash, depth, alpha, beta);

        if let Some(s) = table_score {
            return (s, true)
        }

        if depth == 0 {
            let score = board.q_search(8, alpha, beta);
            self.table.record(board, score, Move::NULL, depth, NodeBound::Exact);
            return (score, true)
        }

        if allow_null && depth >= 3 && !board.is_in_check() {
            let r = if depth > 6 { 3 } else { 2 };
            let mut new_board = *board;
            new_board.move_num += 1;
            new_board.hash.flip_color();
            new_board.hash.set_ep(new_board.en_passant);
            new_board.en_passant = 0;
            let (s, _) = self.negamax_a_b(&new_board, depth - r - 1, -beta, -beta + 1, false);
            if -s >= beta { return (-s, true) }
        }

        let mut has_legal_move = false;

        let moves = board.sort_with(&mut board.get_moves(), best_move,
                                    &self.killers[(self.cur_depth - depth) as usize]);

        for (_, mv) in moves {
            let mut new_board = *board;
            new_board.make_move(mv);

            let (mut score, is_legal) = self.negamax_a_b(&new_board, depth - 1, -beta, -alpha, true);
            score *= -1;

            if is_legal { has_legal_move = true; } else { continue }

            if score >= beta {
                if !mv.is_capture() { self.killers[(self.cur_depth - depth) as usize].substitute(mv) }
                self.table.record(board, score, mv, depth, NodeBound::Beta);
                return (score, true)
            }
            if score > alpha {
                best_move = mv;
                alpha = score;
            }
        }

        if !has_legal_move {
            if board.is_in_check() {
                return (-1000000 - depth as i32, true)
            } else {
                return (0, true)
            }
        }

        self.table.record(board, alpha, best_move, depth, NodeBound::Alpha);
        (alpha, true)
    }

    pub fn pv_search(&mut self, board: &Board, depth: u8, mut alpha: i32, beta: i32) -> (i32, bool) {
        if board.player_in_check(board.prev_move()) { return (0, false) }

        let (table_score, mut best_move) = self.table.probe(board.hash, depth, alpha, beta);

        if let Some(s) = table_score {
            return (s, true)
        }

        if depth == 0 {
            let score = board.q_search(8, alpha, beta);
            self.table.record(board, score, Move::NULL, depth, NodeBound::Exact);
            return (score, true)
        }

        let mut has_legal_move = false;

        let moves = board.sort_with(&mut board.get_moves(), best_move,
                                    &self.killers[(self.cur_depth - depth) as usize]);

        let mut first = true;

        for (_, mv) in moves {
            let mut new_board = *board;
            new_board.make_move(mv);

            let (mut score, is_legal) = if !first {
                let (s, l) = self.pv_search(&new_board, depth - 1, -(alpha + 1), -alpha);
                if -s > alpha && -s < beta {
                    self.pv_search(&new_board, depth - 1, -beta, s)
                } else {
                    (s, l)
                }
            } else {
                first = false;
                self.pv_search(&new_board, depth - 1, -beta, -alpha)
            };

            score *= -1;

            if is_legal { has_legal_move = true; } else { continue }

            if score >= beta {
                if !mv.is_capture() { self.killers[(self.cur_depth - depth) as usize].substitute(mv) }
                self.table.record(board, score, mv, depth, NodeBound::Beta);
                return (score, true)
            }
            if score > alpha {
                best_move = mv;
                alpha = score;
            }
        }

        if !has_legal_move {
            if board.is_in_check() {
                return (-1000000 - depth as i32, true)
            } else {
                return (0, true)
            }
        }

        self.table.record(board, alpha, best_move, depth, NodeBound::Alpha);
        (alpha, true)
    }
}

impl Board {
    pub fn q_search(&self, depth: u8, mut alpha: i32, beta: i32) -> i32 {
        if self.player_in_check(self.prev_move()) { return 10000000 }
        // TODO: remove depth so all takes are searched
        // TODO: When no legal moves possible, return draw to avoid stalemate
        // TODO: Three move repition
        let stand_pat = self.evaluate();
        if depth == 0 || stand_pat >= beta { return stand_pat }
        if stand_pat > alpha { alpha = stand_pat }

        let mut captures = self.get_moves();
        captures.retain(|mv| mv.is_capture());

        for (_, mv) in self.sort(&captures) {
            let mut new_board = *self;
            new_board.make_move(mv);
            let score = -new_board.q_search(depth - 1, -beta, -alpha);

            if score >= beta { return score }
            if score > alpha { alpha = score; }
        }
        alpha
    }
}
