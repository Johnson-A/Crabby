use types::*;
use util::*;
use magics::*;

impl Board {
    pub fn get_evals(us: &BitBoard, opp: &BitBoard) -> i32 {
        // TODO: remove king material? With legal move checking, and mate and stalemate now added
        let occ = us.pieces | opp.pieces;

        let mut eval = 0;

        for_all_pieces(us.queen, &mut |from| {
            let att = unsafe { BISHOP_MAP[from as usize].att(occ) |
                               ROOK_MAP[from as usize].att(occ) };
            eval += (att & !occ).count_ones() * 5 +
                    (att & opp.pieces).count_ones() * 15 +
                    (att & us.pieces).count_ones() * 8;
        });

        for_all_pieces(us.rook, &mut |from| {
            let att = unsafe { ROOK_MAP[from as usize].att(occ) };
            eval += (att & !occ).count_ones() * 15 +
                    (att & opp.pieces).count_ones() * 20 +
                    (att & us.pieces).count_ones() * 10;
        });

        for_all_pieces(us.bishop, &mut |from| {
            let att = unsafe { BISHOP_MAP[from as usize].att(occ) };
            eval += (att & !occ).count_ones() * 25 +
                    (att & opp.pieces).count_ones() * 30 +
                    (att & us.pieces).count_ones() * 10;
        });

        for_all_pieces(us.knight, &mut |from| {
            let att = unsafe { KNIGHT_MAP[from as usize] };
            eval += (att & !occ).count_ones() * 30 +
                    (att & opp.pieces).count_ones() * 35 +
                    (att & us.pieces).count_ones() * 12;
        });

        for_all_pieces(us.king, &mut |from| {
            let att = unsafe { KING_MAP[from as usize] };
            eval += (att & !occ).count_ones() * 10 +
                    (att & opp.pieces).count_ones() * 15 +
                    (att & us.pieces).count_ones() * 10;
        });

        let material =  (us.pawn.count_ones()   * 1000)  +
                        (us.knight.count_ones() * 3000)  +
                        (us.bishop.count_ones() * 3000)  +
                        (us.rook.count_ones()   * 5000)  +
                        (us.queen.count_ones()  * 9000)  +
                        (us.king.count_ones()   * 300000);

        (material + eval) as i32
    }

    pub fn evaluate(&self) -> i32 {
        // TODO: Don't trade if material down or in worse position
        // TODO: doubled pawns
        // TODO: Center squares and pawns
        let opp = self.prev_move();
        let us = self.to_move(); // Node player

        let mut eval = 1000*1000;

        let is_white = self.is_white();
        let occ = us.pieces | opp.pieces;

        if is_white {
            eval -= ((us.pieces ^ (us.king | us.queen)) & ROW_1).count_ones() * 50;
            eval += ((opp.pieces ^ (opp.king | opp.queen)) & ROW_8).count_ones() * 50;

            let pushes = (us.pawn << 8) & !occ;
            let double_pushes = ((pushes & ROW_3) << 8) & !occ;
            let left_attacks = (us.pawn << 7) & (opp.pieces | self.en_passant) & !FILE_H;
            let right_attacks = (us.pawn << 9) & (opp.pieces | self.en_passant) & !FILE_A;

            eval += pushes.count_ones() * 10 +
                    double_pushes.count_ones() * 10;
            eval += left_attacks.count_ones() * 40 +
                    right_attacks.count_ones() * 40;

            let pushes = (opp.pawn >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (opp.pawn >> 7) & (us.pieces | self.en_passant) & !FILE_A;
            let right_attacks = (opp.pawn >> 9) & (us.pieces | self.en_passant) & !FILE_H;

            eval -= pushes.count_ones() * 10 +
                    double_pushes.count_ones() * 10;
            eval -= left_attacks.count_ones() * 40 +
                    right_attacks.count_ones() * 40;
        } else {
            eval -= ((us.pieces ^ (us.king | us.queen)) & ROW_8).count_ones() * 50;
            eval += ((opp.pieces ^ (opp.king | opp.queen)) & ROW_1).count_ones() * 50;

            let pushes = (us.pawn >> 8) & !occ;
            let double_pushes = ((pushes & ROW_6) >> 8) & !occ;
            let left_attacks = (us.pawn >> 7) & (opp.pieces | self.en_passant) & !FILE_A;
            let right_attacks = (us.pawn >> 9) & (opp.pieces | self.en_passant) & !FILE_H;

            eval += pushes.count_ones() * 10 +
                    double_pushes.count_ones() * 10;
            eval += left_attacks.count_ones() * 40 +
                    right_attacks.count_ones() * 40;

            let pushes = (opp.pawn << 8) & !occ;
            let double_pushes = ((pushes & ROW_3) << 8) & !occ;
            let left_attacks = (opp.pawn << 7) & (us.pieces | self.en_passant) & !FILE_H;
            let right_attacks = (opp.pawn << 9) & (us.pieces | self.en_passant) & !FILE_A;

            eval -= pushes.count_ones() * 10 +
                    double_pushes.count_ones() * 10;
            eval -= left_attacks.count_ones() * 40 +
                    right_attacks.count_ones() * 40;
        }

        let real_eval = (eval as i32) - 1000*1000;
        real_eval + Board::get_evals(us, opp) - Board::get_evals(opp, us)
    }
}
