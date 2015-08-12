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
            let (score, _) = self.negamax_a_b(self.board, depth, -i32::MAX, i32::MAX);

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
    pub fn negamax_a_b(&mut self, board: &Board, depth: u8, mut alpha: i32, beta: i32) -> (i32, bool) {
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
        let enemy_king = board.bb[KING | board.prev_move()].trailing_zeros();
        let mut moves = board.get_moves();

        for mv in &moves {
            if mv.to() == enemy_king { return (0, false) }
        }

        let moves = board.sort_with(&mut moves, best_move, &self.killers[(self.cur_depth - depth) as usize]);

        for (_, mv) in moves {
            let mut new_board = *board;
            new_board.make_move(mv);

            let (mut score, is_legal) = self.negamax_a_b(&new_board, depth - 1, -beta, -alpha);
            score *= -1;

            if is_legal { has_legal_move = true; } else { continue }

            if score >= beta {
                if !mv.is_capture() { self.killers[(self.cur_depth - depth) as usize].substitute(mv) }
                self.table.record(board, score, mv, depth, NodeBound::Beta); // score or beta?
                return (beta, true)
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
        // TODO: remove depth so all takes are searched
        // TODO: Check for king attacks and break for that branch to avoid illegal moves
        // TODO: When no legal moves possible, return draw to avoid stalemate
        // TODO: Three move repition
        // TODO: Add illegal move detection in queiscence which might otherwise cause subtle bugs
        let stand_pat = self.evaluate();
        if depth == 0 { return stand_pat }
        if stand_pat >= beta { return beta }
        if stand_pat > alpha { alpha = stand_pat }

        let mut captures = self.get_moves();
        captures.retain(|mv| mv.is_capture());

        for (_, mv) in self.sort(&captures) {
            let mut new_board = *self;
            new_board.make_move(mv);
            let score = -new_board.q_search(depth - 1, -beta, -alpha);

            if score >= beta { return beta }
            if score > alpha { alpha = score; }
        }
        alpha
    }
}
