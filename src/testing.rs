#![cfg(test)]
use test::Bencher;
use types::*;
use magics::*;
use uci;

static mut INITIALIZED: bool = false;

pub fn check_init() {
    unsafe {
        if !INITIALIZED {
            uci::init();
            INITIALIZED = true;
        }
    }
}

#[bench]
pub fn move_gen(b: &mut Bencher) {
    check_init();
    let board = Board::start_position();

    b.iter(|| board.get_moves());
}

// #[bench]
// pub fn get_moves(b: &mut Bencher) {
//     check_init();
//     let mut res = 0;
//     let c = 0;
//
//     b.iter(|| black_box({
//         res |= bishop_moves(0, c);
//         res |= bishop_moves(0, c);
//         res |= bishop_moves(0, c);
//         res |= bishop_moves(0, c);
//         res |= bishop_moves(0, c);
//
//         res |= rook_moves(0, c);
//         res |= rook_moves(0, c);
//         res |= rook_moves(0, c);
//         res |= rook_moves(0, c);
//         res |= rook_moves(0, c);
//     }));
// }
