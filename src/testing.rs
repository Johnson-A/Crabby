#![cfg(test)]
use test::Bencher;
use magics::*;
use board::Board;
use uci;

#[bench]
pub fn a_move_gen(b: &mut Bencher) {
    // This is to ensure the initialization has been called already
    if bishop_moves(0, 0) == 0 { uci::init(); }

    let board = Board::start_position();

    b.iter(|| board.get_moves());
}

#[bench]
pub fn b_get_moves(b: &mut Bencher) {
    let mut res = 0;
    let c = 0;

    b.iter(|| {
        res |= bishop_moves(0, c);
        res |= bishop_moves(0, c);
        res |= bishop_moves(0, c);
        res |= bishop_moves(0, c);
        res |= bishop_moves(0, c);

        res |= rook_moves(0, c);
        res |= rook_moves(0, c);
        res |= rook_moves(0, c);
        res |= rook_moves(0, c);
        res |= rook_moves(0, c);
    });
    if res == 0 {
        println!("{}", res);
    }
}

#[bench]
pub fn eval_speed(b: &mut Bencher) {
    let board = Board::start_position();

    b.iter(|| board.evaluate());
}
