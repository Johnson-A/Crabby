use std::fmt;
use piece::*;


const ROW_1: u64 = 0x00000000000000FF;
const ROW_2: u64 = ROW_1 << 8;
const ROW_3: u64 = ROW_1 << 8 * 2;
const ROW_4: u64 = ROW_1 << 8 * 3;
const ROW_5: u64 = ROW_1 << 8 * 4;
const ROW_6: u64 = ROW_1 << 8 * 5;
const ROW_7: u64 = ROW_1 << 8 * 6;
const ROW_8: u64 = ROW_1 << 8 * 7;

const FILE_A: u64 = 0x0101010101010101;
const FILE_B: u64 = FILE_A << 1;
const FILE_C: u64 = FILE_A << 2;
const FILE_D: u64 = FILE_A << 3;
const FILE_E: u64 = FILE_A << 4;
const FILE_F: u64 = FILE_A << 5;
const FILE_G: u64 = FILE_A << 6;
const FILE_H: u64 = FILE_A << 7;

const BIT_REV: [u64; 256] = [
  0x00, 0x80, 0x40, 0xC0, 0x20, 0xA0, 0x60, 0xE0, 0x10, 0x90, 0x50, 0xD0, 0x30, 0xB0, 0x70, 0xF0, 
  0x08, 0x88, 0x48, 0xC8, 0x28, 0xA8, 0x68, 0xE8, 0x18, 0x98, 0x58, 0xD8, 0x38, 0xB8, 0x78, 0xF8, 
  0x04, 0x84, 0x44, 0xC4, 0x24, 0xA4, 0x64, 0xE4, 0x14, 0x94, 0x54, 0xD4, 0x34, 0xB4, 0x74, 0xF4, 
  0x0C, 0x8C, 0x4C, 0xCC, 0x2C, 0xAC, 0x6C, 0xEC, 0x1C, 0x9C, 0x5C, 0xDC, 0x3C, 0xBC, 0x7C, 0xFC, 
  0x02, 0x82, 0x42, 0xC2, 0x22, 0xA2, 0x62, 0xE2, 0x12, 0x92, 0x52, 0xD2, 0x32, 0xB2, 0x72, 0xF2, 
  0x0A, 0x8A, 0x4A, 0xCA, 0x2A, 0xAA, 0x6A, 0xEA, 0x1A, 0x9A, 0x5A, 0xDA, 0x3A, 0xBA, 0x7A, 0xFA,
  0x06, 0x86, 0x46, 0xC6, 0x26, 0xA6, 0x66, 0xE6, 0x16, 0x96, 0x56, 0xD6, 0x36, 0xB6, 0x76, 0xF6, 
  0x0E, 0x8E, 0x4E, 0xCE, 0x2E, 0xAE, 0x6E, 0xEE, 0x1E, 0x9E, 0x5E, 0xDE, 0x3E, 0xBE, 0x7E, 0xFE,
  0x01, 0x81, 0x41, 0xC1, 0x21, 0xA1, 0x61, 0xE1, 0x11, 0x91, 0x51, 0xD1, 0x31, 0xB1, 0x71, 0xF1,
  0x09, 0x89, 0x49, 0xC9, 0x29, 0xA9, 0x69, 0xE9, 0x19, 0x99, 0x59, 0xD9, 0x39, 0xB9, 0x79, 0xF9, 
  0x05, 0x85, 0x45, 0xC5, 0x25, 0xA5, 0x65, 0xE5, 0x15, 0x95, 0x55, 0xD5, 0x35, 0xB5, 0x75, 0xF5,
  0x0D, 0x8D, 0x4D, 0xCD, 0x2D, 0xAD, 0x6D, 0xED, 0x1D, 0x9D, 0x5D, 0xDD, 0x3D, 0xBD, 0x7D, 0xFD,
  0x03, 0x83, 0x43, 0xC3, 0x23, 0xA3, 0x63, 0xE3, 0x13, 0x93, 0x53, 0xD3, 0x33, 0xB3, 0x73, 0xF3, 
  0x0B, 0x8B, 0x4B, 0xCB, 0x2B, 0xAB, 0x6B, 0xEB, 0x1B, 0x9B, 0x5B, 0xDB, 0x3B, 0xBB, 0x7B, 0xFB,
  0x07, 0x87, 0x47, 0xC7, 0x27, 0xA7, 0x67, 0xE7, 0x17, 0x97, 0x57, 0xD7, 0x37, 0xB7, 0x77, 0xF7, 
  0x0F, 0x8F, 0x4F, 0xCF, 0x2F, 0xAF, 0x6F, 0xEF, 0x1F, 0x9F, 0x5F, 0xDF, 0x3F, 0xBF, 0x7F, 0xFF
];

fn reverse(bits: u64) -> u64 {
	(BIT_REV[(bits & 0xff) as usize]       	 << 56) | 
	(BIT_REV[((bits >> 8)  & 0xff) as usize] << 48) | 
	(BIT_REV[((bits >> 16) & 0xff) as usize] << 40) |
	(BIT_REV[((bits >> 24) & 0xff) as usize] << 32) |
	(BIT_REV[((bits >> 32) & 0xff) as usize] << 24) | 
	(BIT_REV[((bits >> 40) & 0xff) as usize] << 16) | 
	(BIT_REV[((bits >> 48) & 0xff) as usize] << 8)  |
	BIT_REV[((bits >> 56) & 0xff) as usize]
}

fn bit_pop(x: &mut u64) -> u32 {
	let lsb = x.trailing_zeros();
	*x ^= 1 << lsb;
	lsb
}

fn file(from: u32) -> u64 {
	FILE_A << (from % 8)
}

fn row(from: u32) -> u64 {
	ROW_1 << (8 * (from / 8))
}

const main_diag: u64 = 0x8040201008040201;
fn diag(from: u32) -> u64 {
	let diag_index = ((from / 8) - (from % 8)) & 15;
	if diag_index <= 7 {main_diag << 8*diag_index} else {main_diag >> 8*(16 - diag_index)}
}

const main_anti_diag: u64 = 0x0102040810204080;
fn anti_diag(from: u32) -> u64 {
	0
}

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