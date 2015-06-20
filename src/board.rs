use std::fmt;
use piece::*;
use util::*;

pub type Squares = [[Square; 8]; 8];

pub fn gen_bitboards(sqs: &Squares) -> (BitBoard, BitBoard) {
	let mut w: BitBoard = Default::default();
	let mut b: BitBoard= Default::default();

	for i in 0..64 {
		match sqs[i/8][i%8] {
			Square::Piece(pt, col) => {
				let bb = if col == Color::White { &mut w } else { &mut b };
				match pt {
					PieceType::Pawn   => bb.pawn |= 1 << i,
					PieceType::Knight => bb.knight |= 1 << i,
					PieceType::Bishop => bb.bishop |= 1 << i,
					PieceType::Rook   => bb.rook |= 1 << i,
					PieceType::Queen  => bb.queen |= 1 << i,
					PieceType::King   => bb.king |= 1 << i
				};
			},
			Square::Empty => continue
		}
	}

	w.pieces = w.pawn | w.knight | w.bishop | w.rook | w.queen | w.king;
	b.pieces = b.pawn | b.knight | b.bishop | b.rook | b.queen | b.king;
	(w, b)
}

#[derive(Debug)]
pub struct Move { // single u32 ?
	pub flags: u16,
	pub from: u8,
	pub to: u8
}

pub fn add_moves(moves: &mut Vec<Move>, mut targets: u64, diff: u8) {
	while targets != 0 {
		let to = bit_pop(&mut targets) as u8;
		let from = to - diff;
		// let capture = board
		let mv = Move {from: from, to: to, flags: 0 };
		moves.push(mv);
	}
}

pub fn get_line_attacks(occ: u64, mask: u64, piece: u64) -> u64 {
	let pot_blockers = occ & mask;
	let forward = (pot_blockers - 2*piece);
	let rev = reverse(reverse(pot_blockers) - 2*reverse(piece));
	(forward ^ rev) & mask
}

pub struct Board {
	pub w: BitBoard,
	pub b: BitBoard,
	pub sqs: Squares
}

impl Board {
	pub fn make_move(&mut self, (sr, sc): Position, (dr, dc): Position) { // TODO:
		self.sqs[dr][dc] = self.sqs[sr][sc];
		self.sqs[sr][sc] = Square::Empty;
		let (w, b) = gen_bitboards(&self.sqs);
		self.w = w;
		self.b = b;
	}

	pub fn make_str_move(&mut self, mv: &str) {
		let moves: Vec<char> = mv.chars().collect();
		match moves.as_slice() {
			[sc, sr, dc, dr] => {
				let src_pos = to_pos(sc, sr);
				let dest_pos = to_pos(dc, dr);
				self.make_move(src_pos, dest_pos);
			},
			_ => return // malformed move
		}
	}

	pub fn get_moves(&self, color: Color) -> Vec<Move> { // TODO:
		let mut moves: Vec<Move> = Vec::with_capacity(32);
		
		let possible_squares = !self.w.pieces;

		let occupied = self.w.pieces | self.b.pieces;
		
		if color == Color::White {
			// Pawn
			let mut pushes = (self.w.pawn << 8) & !occupied;

			let double_pushes = ((pushes & ROW_3) << 8) & !occupied;

			let left_attacks = (self.w.pawn << 7) & self.b.pieces & !FILE_H;

			let right_attacks = (self.w.pawn << 9) & self.b.pieces & !FILE_A;

			let promotions = pushes & ROW_8; // Get all moves
			// let promotion_captures = (left_attacks | right_attacks) & ROW_8;
			pushes &= !ROW_8; // Remove ROW_8 pushes

			// En Pessant goes here
			add_moves(&mut moves, pushes, 8);
			add_moves(&mut moves, double_pushes, 16);
			add_moves(&mut moves, left_attacks, 7);
			add_moves(&mut moves, right_attacks, 9);
			add_moves(&mut moves, promotions, 8);

			let mut queen_bb = self.w.queen;
			while queen_bb != 0 {
				let from = bit_pop(&mut queen_bb);
				let piece = 1 << from;

				let attacks = 	get_line_attacks(occupied, file(from), piece) |
								get_line_attacks(occupied, row(from), piece)  |
								get_line_attacks(occupied, diag(from), piece);

				let mut qmoves = attacks & !self.w.pieces;

				while qmoves != 0 {
					let to = bit_pop(&mut qmoves);
					moves.push(Move {from: from as u8, to: to as u8, flags: 0});
				}
			}

			let mut rook_bb = self.w.rook;
			while rook_bb != 0 {
				let from = bit_pop(&mut rook_bb);
				let piece = 1 << from;

				let attacks = 	get_line_attacks(occupied, file(from), piece) |
								get_line_attacks(occupied, row(from), piece);

				let mut rmoves = attacks & !self.w.pieces;

				while rmoves != 0 {
					let to = bit_pop(&mut rmoves);
					moves.push(Move {from: from as u8, to: to as u8, flags: 0});
				}
			}

			let mut bishop_bb = self.w.bishop;
			while bishop_bb != 0 {
				let from = bit_pop(&mut bishop_bb);
				let piece = 1 << from;

				let attacks = 	get_line_attacks(occupied, diag(from), piece);

				let mut bmoves = attacks & !self.w.pieces;

				while bmoves != 0 {
					let to = bit_pop(&mut bmoves);
					moves.push(Move {from: from as u8, to: to as u8, flags: 0});
				}
			}

		} else {

		}

					// Knight
			// let mut rem_knights = self.w.knight;

			// while rem_knights != 0 {
			// 	let pos = rem_knights & (-rem_knights); // v & -v
			// 	// moves.push()
			// 	rem_knights ^= pos;
			// }

		moves
	}

	pub fn new(fen_board: &str) -> Board {
		let reversed_rows = fen_board.split('/').rev(); // fen is read from top rank
		let mut sqs = [[Square::Empty; 8]; 8];

		for (r, row) in reversed_rows.enumerate() {
			for (c, ch) in row.chars().enumerate() {
				if !ch.is_numeric() {
					sqs[r][c] = to_piece(ch);
				}
			}
		}
		let (w, b) = gen_bitboards(&sqs);

		Board { w: w, b: b, sqs: sqs }
	}
}

impl fmt::Display for Board {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		let mut characters = Vec::new();

		for row in self.sqs.iter().rev() {
			for sq in row.iter() {
				characters.push(to_char(sq));
			}
			characters.push('\n');
		}
		let output = characters.iter().cloned().collect::<String>();
		write!(f, "{}", output)
	}
}

#[derive(Debug, Default)]
pub struct BitBoard {
	pub pawn: u64,
	pub knight: u64,
	pub bishop: u64,
	pub rook: u64,
	pub queen: u64,
	pub king: u64,
	pub pieces: u64
}