use std::cmp::Ordering::{Less, Greater};
use types::*;

impl Board {
	pub fn q_search(&self, depth: u32, mut alpha: i32, beta: i32) -> i32 {
		// TODO: remove depth so all takes are searched
		// TODO: Check for king attacks and break for that branch to avoid illegal moves
		// TODO: When no legal moves possible, return draw to avoid stalemate
		// TODO: Three move repition
		// TODO: Add illegal move detection in queiscence which might cause subtle bugs
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

	// TODO: Fail soft
	pub fn negamax_a_b(&self, depth: u32, mut alpha: i32, beta: i32, line: &mut Vec<Move>) -> (i32, bool) {
		if depth == 0 { return (self.q_search(4, alpha, beta), true) }
		let mut has_legal_move = false;
		let enemy_king = self.prev_move().king.trailing_zeros();
		let mut localpv = Vec::new();

		for mv in self.get_moves() {
			if mv.to() == enemy_king { return (0, false) }
			let mut new_board = self.clone();
			new_board.make_move(mv);

			let (mut score, is_legal) = new_board.negamax_a_b(depth - 1, -beta, -alpha, &mut localpv);
			score *= -1;

			if is_legal { has_legal_move = true; } else { continue }

			if score >= beta { return (beta, true) }
			if score > alpha {
				alpha = score;
				line.clear();
				line.push(mv);
				line.append(&mut localpv);
			}
		}

		if !has_legal_move {
			if self.is_in_check() {
				return (-1000000 - depth as i32, true)
			} else {
				return (0, true)
			}
		}

		(alpha, true)
	}

	pub fn is_in_check(&self) -> bool {
		let king_pos = self.to_move().king.trailing_zeros();

		// TODO: Board needs to be mutable to avoid clone here
		let mut clone = self.clone();
		clone.move_num += 1;

		for mv in clone.get_moves() { // get opponent moves
			if mv.to() == king_pos { return true }
		}
		false
	}

	pub fn iter_deep(&self, depth: u32, moves: Vec<Move>) {
		let mut pv = Vec::new();
		let mut scores = Vec::new();

		for mv in moves {
			let mut new_board = self.clone();
			new_board.make_move(mv);
			let (score, _) = new_board.negamax_a_b(depth, -100000, 100000, &mut pv);
			scores.push((mv, -score));
		}

		scores.sort_by(|a, b|
			if a.1 >= b.1 { Less } else { Greater }
		);
		println!("{} {}", depth, scores.iter().map(|a| a.0.to_str() + " " + &a.1.to_string()).collect::<Vec<_>>().connect(" "));
		let sorted_moves = scores.iter().map(|a| a.0 ).collect();

		self.iter_deep(depth + 1, sorted_moves);
	}
}
