use types::*;
use table::*;

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

        for mv in self.get_moves().into_iter().filter(|mv| mv.is_capture()) {
            let mut new_board = self.clone();
            new_board.make_move(mv);
            let score = -new_board.q_search(depth - 1, -beta, -alpha);

            if score >= beta { return beta }
            if score > alpha { alpha = score; }
        }
        alpha
    }

    // TODO: Fail soft, retain the pv
    pub fn negamax_a_b(&self, depth: u8, mut alpha: i32, beta: i32, table: &mut Table) -> (i32, bool) {
        let (table_score, mut best_move) = table.probe(self.hash, depth, alpha, beta);

        match table_score {
            Some(s) => return (s, true),
            None => ()
        }

        if depth == 0 {
            let score = self.q_search(8, alpha, beta);
            table.record(self, score, Move::NULL, depth, NodeBound::Exact);
            return (score, true)
        }

        let mut has_legal_move = false;
        let enemy_king = self.bb[KING | self.prev_move()].trailing_zeros();

        let mut moves = self.get_moves();

        for mv in &moves {
            if mv.to() == enemy_king { return (0, false) }
        }

        if best_move != Move::NULL {
            let ind = moves.iter().position(|x| *x == best_move);
            match ind {
                Some(val) => {
                    moves.remove(val);
                    moves.insert(0, best_move);
                },
                None => println!("UHOH")
            }
        }

        for mv in moves {
            let mut new_board = self.clone();
            new_board.make_move(mv);

            let (mut score, is_legal) = new_board.negamax_a_b(depth - 1, -beta, -alpha, table);
            score *= -1;

            if is_legal { has_legal_move = true; } else { continue }

            if score >= beta {
                table.record(self, beta, mv, depth, NodeBound::Beta);
                return (beta, true)
            }
            if score > alpha {
                best_move = mv;
                alpha = score;
            }
        }

        if !has_legal_move {
            if self.is_in_check() {
                return (-1000000 - depth as i32, true)
            } else {
                return (0, true)
            }
        }

        table.record(self, alpha, best_move, depth, NodeBound::Alpha);
        (alpha, true)
    }

    pub fn is_in_check(&self) -> bool {
        let king_pos = self.bb[KING | self.to_move()].trailing_zeros();

        // TODO: Board needs to be mutable to avoid clone here
        let mut clone = self.clone();
        clone.move_num += 1;

        for mv in clone.get_moves() { // get opponent moves
            if mv.to() == king_pos { return true }
        }
        false
    }
}
