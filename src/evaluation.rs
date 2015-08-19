use types::*;
use util::*;
use magics::*;

impl Board {
    // Attack map by square
    // Piece Values by Square
    // King safety
    // Doubled pawns
    // Attacks defends - who owns more squares
    // Space advantage
    // Simplify when ahead
    pub fn get_evals(&self, us: u8, opp: u8) -> i32 {
        // TODO: remove king material? With legal move checking, and mate and stalemate now added - qsearch still
        let bb = &self.bb;
        let occ = bb[ALL | us] | bb[ALL | opp];

        let mut eval = 0;

        for_all(bb[QUEEN | us], &mut |from| {
            let att = queen_moves(from, occ);
            eval += (att & !occ).count_ones() * 5 +
                    (att & bb[ALL | opp]).count_ones() * 15 +
                    (att & bb[ALL | us] ).count_ones() * 8;
        });

        for_all(bb[ROOK | us], &mut |from| {
            let att = rook_moves(from, occ);
            eval += (att & !occ).count_ones() * 15 +
                    (att & bb[ALL | opp]).count_ones() * 20 +
                    (att & bb[ALL | us] ).count_ones() * 10;
        });

        for_all(bb[BISHOP | us], &mut |from| {
            let att = bishop_moves(from, occ);
            eval += (att & !occ).count_ones() * 25 +
                    (att & bb[ALL | opp]).count_ones() * 30 +
                    (att & bb[ALL | us] ).count_ones() * 10;
        });

        for_all(bb[KNIGHT | us], &mut |from| {
            let att = knight_moves(from);
            eval += (att & !occ).count_ones() * 30 +
                    (att & bb[ALL | opp]).count_ones() * 35 +
                    (att & bb[ALL | us] ).count_ones() * 12;
        });

        for_all(bb[KING | us], &mut |from| {
            let att = king_moves(from);
            eval += (att & !occ).count_ones() * 10 +
                    (att & bb[ALL | opp]).count_ones() * 15 +
                    (att & bb[ALL | us] ).count_ones() * 10;
        });

        let material = (bb[PAWN   | us].count_ones() * p_val(PAWN))   +
                       (bb[KNIGHT | us].count_ones() * p_val(KNIGHT)) +
                       (bb[BISHOP | us].count_ones() * p_val(BISHOP)) +
                       (bb[ROOK   | us].count_ones() * p_val(ROOK))   +
                       (bb[QUEEN  | us].count_ones() * p_val(QUEEN))  +
                       (bb[KING   | us].count_ones() * p_val(KING));

        (material + eval) as i32
    }

    pub fn evaluate(&self) -> i32 {
        // TODO: Don't trade if material down or in worse position
        // TODO: doubled pawns
        // TODO: Center squares and pawns
        let bb = &self.bb;
        let us = self.to_move(); // Node player
        let opp = self.prev_move();

        let mut eval = 1000*1000;

        let occ = bb[ALL | us] | bb[ALL | opp];;

        if us == WHITE {
            eval -= ((bb[ALL | us] ^ (bb[KING | us] | bb[QUEEN | us])) & ROW_1).count_ones() * 50;
            eval += ((bb[ALL | opp] ^ (bb[KING | opp] | bb[QUEEN | opp])) & ROW_8).count_ones() * 50;

            let pushes = (bb[PAWN | us] << 8) & !occ;
            let double_pushes = ((pushes & ROW_3) << 8) & !occ;
            let left_attacks = (bb[PAWN | us] << 7) & (bb[ALL | opp] | self.en_passant) & !FILE_H;
            let right_attacks = (bb[PAWN | us] << 9) & (bb[ALL | opp] | self.en_passant) & !FILE_A;

            eval += pushes.count_ones() * 10 +
                    double_pushes.count_ones() * 10;
            eval += left_attacks.count_ones() * 40 +
                    right_attacks.count_ones() * 40;

            let pushes = (bb[PAWN | opp] >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (bb[PAWN | opp] >> 7) & (bb[ALL | us] | self.en_passant) & !FILE_A;
            let right_attacks = (bb[PAWN | opp] >> 9) & (bb[ALL | us] | self.en_passant) & !FILE_H;

            eval -= pushes.count_ones() * 10 +
                    double_pushes.count_ones() * 10;
            eval -= left_attacks.count_ones() * 40 +
                    right_attacks.count_ones() * 40;
        } else {
            eval -= ((bb[ALL | us] ^ (bb[KING | us] | bb[QUEEN | us])) & ROW_8).count_ones() * 50;
            eval += ((bb[ALL | opp] ^ (bb[KING | opp] | bb[QUEEN | opp])) & ROW_1).count_ones() * 50;

            let pushes = (bb[PAWN | us] >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (bb[PAWN | us] >> 7) & (bb[ALL | opp] | self.en_passant) & !FILE_A;
            let right_attacks = (bb[PAWN | us] >> 9) & (bb[ALL | opp] | self.en_passant) & !FILE_H;

            eval += pushes.count_ones() * 10 +
                    double_pushes.count_ones() * 10;
            eval += left_attacks.count_ones() * 40 +
                    right_attacks.count_ones() * 40;

            let pushes = (bb[PAWN | opp] << 8) & !occ;
            let double_pushes = ((pushes & ROW_3) << 8) & !occ;
            let left_attacks = (bb[PAWN | opp] << 7) & (bb[ALL | us] | self.en_passant) & !FILE_H;
            let right_attacks = (bb[PAWN | opp] << 9) & (bb[ALL | us] | self.en_passant) & !FILE_A;

            eval -= pushes.count_ones() * 10 +
                    double_pushes.count_ones() * 10;
            eval -= left_attacks.count_ones() * 40 +
                    right_attacks.count_ones() * 40;
        }

        let real_eval = (eval as i32) - 1000*1000;
        real_eval + self.get_evals(us, opp) - self.get_evals(opp, us)
    }
}
