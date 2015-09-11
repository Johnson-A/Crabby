use types::*;
use util::*;
use magics::*;

pub static SAFE_MASK: [u64; 2] = [
(FILE_C | FILE_D | FILE_E | FILE_F) & (ROW_7 | ROW_6 | ROW_5),
(FILE_C | FILE_D | FILE_E | FILE_F) & (ROW_2 | ROW_3 | ROW_4)
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

        for_all(bb[QUEEN | us], &mut |from| {
            let att = queen_moves(from, occ);
            eval += (att & !occ).count_ones() * 5 +
                    (att & enemies).count_ones() * 15 +
                    (att & allies).count_ones() * 8;
            attacked_by[QUEEN | us] |= att;
        });

        for_all(bb[ROOK | us], &mut |from| {
            let att = rook_moves(from, occ);
            eval += (att & !occ).count_ones() * 15 +
                    (att & enemies).count_ones() * 20 +
                    (att & allies).count_ones() * 15;
            attacked_by[ROOK | us] |= att;
        });

        for_all(bb[BISHOP | us], &mut |from| {
            let att = bishop_moves(from, occ);
            eval += (att & !occ).count_ones() * 17 +
                    (att & enemies).count_ones() * 30 +
                    (att & allies).count_ones() * 15;
            attacked_by[BISHOP | us] |= att;
        });

        for_all(bb[KNIGHT | us], &mut |from| {
            let att = knight_moves(from);
            eval += (att & !occ).count_ones() * 20 +
                    (att & enemies).count_ones() * 35 +
                    (att & allies).count_ones() * 15;
            attacked_by[KNIGHT | us] |= att;
        });

        for_all(bb[KING | us], &mut |from| {
            let att = king_moves(from);
            eval += (att & !occ).count_ones() * 4 +
                    (att & enemies).count_ones() * 15 +
                    (att & allies).count_ones() * 10;
            attacked_by[KING | us] |= att;
        });

        let material = (bb[PAWN   | us].count_ones() * p_val(PAWN))   +
                       (bb[KNIGHT | us].count_ones() * p_val(KNIGHT)) +
                       (bb[BISHOP | us].count_ones() * p_val(BISHOP)) +
                       (bb[ROOK   | us].count_ones() * p_val(ROOK))   +
                       (bb[QUEEN  | us].count_ones() * p_val(QUEEN))  +
                       (bb[KING   | us].count_ones() * p_val(KING));

        (material + eval) as i32
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
            eval -= ((bb[KNIGHT | us] | bb[BISHOP | us])   & ROW_1).count_ones() * 50;
            eval += ((bb[KNIGHT | opp] | bb[BISHOP | opp]) & ROW_8).count_ones() * 50;

            let pushes = (bb[PAWN | us] << 8) & !occ;
            let double_pushes = ((pushes & ROW_3) << 8) & !occ;
            let left_attacks = (bb[PAWN | us] << 7) & (bb[ALL | opp] | self.en_passant) & !FILE_H;
            let right_attacks = (bb[PAWN | us] << 9) & (bb[ALL | opp] | self.en_passant) & !FILE_A;
            attacked_by[PAWN | us] |= left_attacks | right_attacks;

            eval += pushes.count_ones() * 10 +
                    double_pushes.count_ones() * 10;
            eval += left_attacks.count_ones() * 30 +
                    right_attacks.count_ones() * 30;

            let pushes = (bb[PAWN | opp] >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (bb[PAWN | opp] >> 7) & (bb[ALL | us] | self.en_passant) & !FILE_A;
            let right_attacks = (bb[PAWN | opp] >> 9) & (bb[ALL | us] | self.en_passant) & !FILE_H;
            attacked_by[PAWN | opp] |= left_attacks | right_attacks;

            eval -= pushes.count_ones() * 10 +
                    double_pushes.count_ones() * 10;
            eval -= left_attacks.count_ones() * 30 +
                    right_attacks.count_ones() * 30;
        } else {
            eval -= ((bb[KNIGHT | us] | bb[BISHOP | us])   & ROW_8).count_ones() * 50;
            eval += ((bb[KNIGHT | opp] | bb[BISHOP | opp]) & ROW_1).count_ones() * 50;

            let pushes = (bb[PAWN | us] >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (bb[PAWN | us] >> 7) & (bb[ALL | opp] | self.en_passant) & !FILE_A;
            let right_attacks = (bb[PAWN | us] >> 9) & (bb[ALL | opp] | self.en_passant) & !FILE_H;
            attacked_by[PAWN | us] |= left_attacks | right_attacks;

            eval += pushes.count_ones() * 10 +
                    double_pushes.count_ones() * 10;
            eval += left_attacks.count_ones() * 30 +
                    right_attacks.count_ones() * 30;

            let pushes = (bb[PAWN | opp] << 8) & !occ;
            let double_pushes = ((pushes & ROW_3) << 8) & !occ;
            let left_attacks = (bb[PAWN | opp] << 7) & (bb[ALL | us] | self.en_passant) & !FILE_H;
            let right_attacks = (bb[PAWN | opp] << 9) & (bb[ALL | us] | self.en_passant) & !FILE_A;
            attacked_by[PAWN | opp] |= left_attacks | right_attacks;

            eval -= pushes.count_ones() * 10 +
                    double_pushes.count_ones() * 10;
            eval -= left_attacks.count_ones() * 30 +
                    right_attacks.count_ones() * 30;
        }

        let diff = self.get_evals(us, opp, &mut attacked_by) - self.get_evals(opp, us, &mut attacked_by);

        attacked_by.set_all();

        eval -= count(attacked_by[ALL | opp] & king_moves(lsb(bb[KING | us]))) * 40;
        eval -= count(attacked_by[ALL | opp] & (1 << lsb(bb[KING | us]))) * 60;

        eval += count(attacked_by[ALL | us] & king_moves(lsb(bb[KING | opp]))) * 40;
        eval += count(attacked_by[ALL | us] & (1 << lsb(bb[KING | opp]))) * 60;

        eval += self.eval_space(us, &mut attacked_by);
        eval -= self.eval_space(opp, &mut attacked_by);

        let real_eval = (eval as i32) - 1000*1000;

        real_eval + diff
    }
}
