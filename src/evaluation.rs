use types::*;
use board::Board;
use bitboard::BitBoard;
use util::*;
use magics::*;

pub static SAFE_MASK: [u64; 2] = [
(FILE_C | FILE_D | FILE_E | FILE_F) & (ROW_7 | ROW_6 | ROW_5),
(FILE_C | FILE_D | FILE_E | FILE_F) & (ROW_2 | ROW_3 | ROW_4)
];

pub static SQUARE_MAP: [usize; 64] =
[56, 57, 58, 59, 60, 61, 62, 63,
48, 49, 50, 51, 52, 53, 54, 55,
40, 41, 42, 43, 44, 45, 46, 47,
32, 33, 34, 35, 36, 37, 38, 39,
24, 25, 26, 27, 28, 29, 30, 31,
16, 17, 18, 19, 20, 21, 22, 23,
8, 9, 10, 11, 12, 13, 14, 15,
0, 1, 2, 3, 4, 5, 6, 7];

fn rel_loc(square: u32, color: u8) -> usize {
    let s = square as usize;
    if color == BLACK { s } else { SQUARE_MAP[s] }
}

// These piece square tables are taken from https://chessprogramming.wikispaces.com/Simplified+evaluation+function
// The sole reason is to avoid writing them out myself! I will be changing them soon, after which
// I will remove this comment
pub static PAWN_SQUARE: [i32; 64] = [
0,  0,  0,  0,  0,  0,  0,  0,
50, 50, 50, 50, 50, 50, 50, 50,
10, 10, 20, 30, 30, 20, 10, 10,
5,  5, 10, 25, 25, 10,  5,  5,
0,  0,  0, 20, 20,  0,  0,  0,
5, -5,-10,  0,  0,-10, -5,  5,
5, 10, 10,-20,-20, 10, 10,  5,
0,  0,  0,  0,  0,  0,  0,  0,
];

pub static KNIGHT_SQUARE: [i32; 64] = [
-50,-40,-30,-30,-30,-30,-40,-50,
-40,-20,  0,  0,  0,  0,-20,-40,
-30,  0, 10, 15, 15, 10,  0,-30,
-30,  5, 15, 20, 20, 15,  5,-30,
-30,  0, 15, 20, 20, 15,  0,-30,
-30,  5, 10, 15, 15, 10,  5,-30,
-40,-20,  0,  5,  5,  0,-20,-40,
-50,-40,-30,-30,-30,-30,-40,-50,
];

pub static BISHOP_SQUARE: [i32; 64] = [
-20,-10,-10,-10,-10,-10,-10,-20,
-10,  0,  0,  0,  0,  0,  0,-10,
-10,  0,  5, 10, 10,  5,  0,-10,
-10,  5,  5, 10, 10,  5,  5,-10,
-10,  0, 10, 10, 10, 10,  0,-10,
-10, 10, 10, 10, 10, 10, 10,-10,
-10,  5,  0,  0,  0,  0,  5,-10,
-20,-10,-10,-10,-10,-10,-10,-20,
];

pub static ROOK_SQUARE: [i32; 64] = [
 0,  0,  0,  0,  0,  0,  0,  0,
 5, 10, 10, 10, 10, 10, 10,  5,
-5,  0,  0,  0,  0,  0,  0, -5,
-5,  0,  0,  0,  0,  0,  0, -5,
-5,  0,  0,  0,  0,  0,  0, -5,
-5,  0,  0,  0,  0,  0,  0, -5,
-5,  0,  0,  0,  0,  0,  0, -5,
 0,  0,  0,  5,  5,  0,  0,  0,
 ];

pub static QUEEN_SQUARE: [i32; 64] = [
-20,-10,-10, -5, -5,-10,-10,-20,
-10,  0,  0,  0,  0,  0,  0,-10,
-10,  0,  5,  5,  5,  5,  0,-10,
 -5,  0,  5,  5,  5,  5,  0, -5,
  0,  0,  5,  5,  5,  5,  0, -5,
-10,  5,  5,  5,  5,  5,  0,-10,
-10,  0,  5,  0,  0,  0,  0,-10,
-20,-10,-10, -5, -5,-10,-10,-20,
];

impl Board {
    // Attack map by square
    // Piece Values by Square
    // King safety
    // Doubled pawns
    // Attacks defends - who owns more squares
    // Simplify when ahead
    // Center squares
    // Middlegame vs endgame
    // Bishop pair
    // Pawn on same color as bishop
    // Trapped bishop
    // Symmetric move generation and evaluation

    pub fn eval_space(&self, us: u8, attacked_by: &mut BitBoard) -> u32 {
        let opp = flip(us);

        let safe =  SAFE_MASK[us as usize]
                  & !self.bb[PAWN | us]
                  & !attacked_by[PAWN | opp]
                  & (attacked_by[ALL | us] | !attacked_by[ALL | opp]);

        let mut behind = self.bb[PAWN | us];
        if us == WHITE {
            behind |= (behind >> 8) | (behind >> 16);
        } else {
            behind |= (behind << 8) | (behind << 16);
        }

        let bonus = count(safe) + count(behind & safe);
        let weight = count(  self.bb[KNIGHT | us]  | self.bb[BISHOP | us]
                           | self.bb[KNIGHT | opp] | self.bb[BISHOP | opp]);

        bonus * weight * weight
    }

    pub fn get_evals(&self, us: u8, opp: u8, attacked_by: &mut BitBoard) -> i32 {
        let bb = &self.bb;
        let allies = bb[ALL | us];
        let enemies = bb[ALL | opp];
        let occ = allies | enemies;

        let mut eval = 0;
        let mut piece_sq = 0;

        for_all(bb[QUEEN | us], &mut |from| {
            let att = queen_moves(from, occ);
            eval += count(att & !occ) * 5 +
                    count(att & enemies) * 15 +
                    count(att & allies) * 8;
            attacked_by[QUEEN | us] |= att;
            piece_sq += QUEEN_SQUARE[rel_loc(from, us)];
        });

        for_all(bb[ROOK | us], &mut |from| {
            let att = rook_moves(from, occ);
            eval += count(att & !occ) * 15 +
                    count(att & enemies) * 20 +
                    count(att & allies) * 15;
            attacked_by[ROOK | us] |= att;
            piece_sq += ROOK_SQUARE[rel_loc(from, us)];
        });

        if count(bb[BISHOP | us]) == 2 { eval += 100 } // Ignore bishop promotions

        for_all(bb[BISHOP | us], &mut |from| {
            let att = bishop_moves(from, occ);
            eval += count(att & !occ) * 17 +
                    count(att & enemies) * 30 +
                    count(att & allies) * 15;
            attacked_by[BISHOP | us] |= att;
            piece_sq += BISHOP_SQUARE[rel_loc(from, us)];
        });

        for_all(bb[KNIGHT | us], &mut |from| {
            let att = knight_moves(from);
            eval += count(att & !occ) * 20 +
                    count(att & enemies) * 35 +
                    count(att & allies) * 15;
            attacked_by[KNIGHT | us] |= att;
            piece_sq += KNIGHT_SQUARE[rel_loc(from, us)];
        });

        for_all(bb[KING | us], &mut |from| {
            let att = king_moves(from);
            eval += count(att & !occ) * 4 +
                    count(att & enemies) * 15 +
                    count(att & allies) * 10;
            attacked_by[KING | us] |= att;
        });

        let material = count(bb[PAWN   | us]) * p_val(PAWN)   +
                       count(bb[KNIGHT | us]) * p_val(KNIGHT) +
                       count(bb[BISHOP | us]) * p_val(BISHOP) +
                       count(bb[ROOK   | us]) * p_val(ROOK)   +
                       count(bb[QUEEN  | us]) * p_val(QUEEN);

        (material + eval) as i32 + piece_sq
    }

    /// Return a static evaluation relative to the player to move in milli-pawns
    pub fn evaluate(&self) -> i32 {
        let bb = &self.bb;
        let us = self.to_move; // Node player
        let opp = self.prev_move();

        let mut attacked_by = BitBoard([0; 14]);

        let mut eval = 1000*1000;

        let occ = bb[ALL | us] | bb[ALL | opp];;

        if us == WHITE {
            eval -= count((bb[KNIGHT | us] | bb[BISHOP | us])   & ROW_1) * 50;
            eval += count((bb[KNIGHT | opp] | bb[BISHOP | opp]) & ROW_8) * 50;

            let pushes = (bb[PAWN | us] << 8) & !occ;
            let double_pushes = ((pushes & ROW_3) << 8) & !occ;
            let left_attacks = (bb[PAWN | us] << 7) & (bb[ALL | opp] | self.en_passant) & !FILE_H;
            let right_attacks = (bb[PAWN | us] << 9) & (bb[ALL | opp] | self.en_passant) & !FILE_A;
            attacked_by[PAWN | us] |= left_attacks | right_attacks;

            eval += count(pushes) * 10 +
                    count(double_pushes) * 10;
            eval += count(left_attacks) * 30 +
                    count(right_attacks) * 30;

            let pushes = (bb[PAWN | opp] >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (bb[PAWN | opp] >> 7) & (bb[ALL | us] | self.en_passant) & !FILE_A;
            let right_attacks = (bb[PAWN | opp] >> 9) & (bb[ALL | us] | self.en_passant) & !FILE_H;
            attacked_by[PAWN | opp] |= left_attacks | right_attacks;

            eval -= count(pushes) * 10 +
                    count(double_pushes) * 10;
            eval -= count(left_attacks) * 30 +
                    count(right_attacks) * 30;
        } else {
            eval -= count((bb[KNIGHT | us] | bb[BISHOP | us])   & ROW_8) * 50;
            eval += count((bb[KNIGHT | opp] | bb[BISHOP | opp]) & ROW_1) * 50;

            let pushes = (bb[PAWN | us] >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (bb[PAWN | us] >> 7) & (bb[ALL | opp] | self.en_passant) & !FILE_A;
            let right_attacks = (bb[PAWN | us] >> 9) & (bb[ALL | opp] | self.en_passant) & !FILE_H;
            attacked_by[PAWN | us] |= left_attacks | right_attacks;

            eval += count(pushes) * 10 +
                    count(double_pushes) * 10;
            eval += count(left_attacks) * 30 +
                    count(right_attacks) * 30;

            let pushes = (bb[PAWN | opp] << 8) & !occ;
            let double_pushes = ((pushes & ROW_3) << 8) & !occ;
            let left_attacks = (bb[PAWN | opp] << 7) & (bb[ALL | us] | self.en_passant) & !FILE_H;
            let right_attacks = (bb[PAWN | opp] << 9) & (bb[ALL | us] | self.en_passant) & !FILE_A;
            attacked_by[PAWN | opp] |= left_attacks | right_attacks;

            eval -= count(pushes) * 10 +
                    count(double_pushes) * 10;
            eval -= count(left_attacks) * 30 +
                    count(right_attacks) * 30;
        }

        let diff = self.get_evals(us, opp, &mut attacked_by) - self.get_evals(opp, us, &mut attacked_by);

        attacked_by.set_all();

        eval -= count(attacked_by[ALL | opp] & king_moves(lsb(bb[KING | us]))) * 40;
        eval -= count(attacked_by[ALL | opp] & (1 << lsb(bb[KING | us]))) * 60;

        eval += count(attacked_by[ALL | us] & king_moves(lsb(bb[KING | opp]))) * 40;
        eval += count(attacked_by[ALL | us] & (1 << lsb(bb[KING | opp]))) * 60;

        eval += self.eval_space(us,  &mut attacked_by);
        eval -= self.eval_space(opp, &mut attacked_by);

        let real_eval = (eval as i32) - 1000*1000;

        real_eval + diff
    }
}
