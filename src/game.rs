use std::fmt;
use piece::*;
use board::*;

pub struct Game {
    pub board: Board,
    pub to_move: Color,
    pub w_castle: bool,
    pub b_castle: bool,
    pub w_time: u32,
    pub b_time: u32,
    pub move_num: u32,
    pub en_pessant: usize,
}

impl fmt::Display for Game {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\n{:?} to move\nWhite Castle = {}, Black Castle = {}\n\
            White time = {}, Black Time = {}\nMove {}",
            self.board, self.to_move, self.w_castle, self.b_castle,
            self.w_time, self.b_time, self.move_num)
    }
}
