use std::fmt;
use piece::*;
use util::*;

pub type Squares = [Square; 64];

pub fn gen_bitboards(sqs: &Squares) -> (BitBoard, BitBoard) {
    let mut w: BitBoard = Default::default();
    let mut b: BitBoard = Default::default();

    for i in 0..64 {
        match sqs[i] {
            Square::Piece(pt, col) => {
                let bb = if col == Color::White { &mut w } else { &mut b };
                match pt {
                    PieceType::Pawn   => bb.pawn   |= 1 << i,
                    PieceType::Knight => bb.knight |= 1 << i,
                    PieceType::Bishop => bb.bishop |= 1 << i,
                    PieceType::Rook   => bb.rook   |= 1 << i,
                    PieceType::Queen  => bb.queen  |= 1 << i,
                    PieceType::King   => bb.king   |= 1 << i
                };
            },
            Square::Empty => continue
        }
    }

    w.pieces = w.pawn | w.knight | w.bishop | w.rook | w.queen | w.king;
    b.pieces = b.pawn | b.knight | b.bishop | b.rook | b.queen | b.king;
    (w, b)
}

#[derive(Copy, Clone, Debug)]
pub struct Move { data: u32 }

impl Move {
    pub fn new(from: u32, to: u32, flags: u32) -> Move {
        let d = from | to << 6 | flags << 12;
        Move { data: d }
    }

    pub fn from(&self)  -> u32 { self.data & 0x3F }
    pub fn to(&self)    -> u32 { (self.data >> 6) & 0x3F }
    pub fn flags(&self) -> u32 { self.data >> 12 }
}

pub fn move_to_str(mv: &Move) -> String {
    let (from, to) = (mv.from() as u8, mv.to() as u8);
    let (sr, sc) = (from / 8, from % 8);
    let (dr, dc) = (to / 8, to % 8);
    let (sr_char, sc_char) = ((sr + b'1') as char, (sc + b'a') as char);
    let (dr_char, dc_char) = ((dr + b'1') as char, (dc + b'a') as char);
    let chars = vec![sc_char, sr_char, dc_char, dr_char];
    chars.into_iter().collect::<String>()
}

pub fn add_moves(moves: &mut Vec<Move>, mut targets: u64, diff: u32) {
    while targets != 0 {
        let to = bit_pop_pos(&mut targets);
        let from = to - diff;
        // let capture = board
        moves.push(Move::new(from, to, 0));
    }
}

pub fn add_moves_from(moves: &mut Vec<Move>, mut targets: u64, from: u32) {
    while targets != 0 {
        let to = bit_pop_pos(&mut targets);
        moves.push(Move::new(from, to, 0));
    }
}

pub fn for_all_pieces(mut pieces: u64, moves: &mut Vec<Move>,
                    attacks: &Fn(u32, u64) -> u64) {
    while pieces != 0 {
        let piece = bit_pop(&mut pieces);
        let from = piece.trailing_zeros();

        let attacks = attacks(from, piece);
        add_moves_from(moves, attacks, from);
    }
}

pub fn get_line_attacks(occ: u64, mask: u64, piece: u64) -> u64 {
    let pot_blockers = occ & mask;
    let forward = pot_blockers - 2*piece;
    let rev = reverse(reverse(pot_blockers) - 2*reverse(piece));
    (forward ^ rev) & mask
}

pub struct Board {
    pub w: BitBoard,
    pub b: BitBoard,
    pub sqs: Squares,
    pub last_move: Move,
    pub last_cap: Square,
    pub to_move: Color,
    pub w_castle: bool,
    pub b_castle: bool,
    pub en_passant: u64
}

impl Board {
    pub fn make_move(&mut self, mv: Move) {
        let (src, dest) = (mv.from() as usize, mv.to() as usize);
        self.last_cap = self.sqs[dest];
        self.sqs[dest] = self.sqs[src];
        self.sqs[src] = Square::Empty;
        let (w, b) = gen_bitboards(&self.sqs);
        self.w = w;
        self.b = b;
        self.to_move.flip();
        self.last_move = mv;
    }

    pub fn unmake(&mut self) {
        let (src, dest) = (self.last_move.from() as usize, self.last_move.to() as usize);
        self.sqs[src] = self.sqs[dest];
        self.sqs[dest] = self.last_cap;
        let (w, b) = gen_bitboards(&self.sqs);
        self.w = w;
        self.b = b;
        self.to_move.flip();
    }

    pub fn make_str_move(&mut self, mv: &str) {
        let moves: Vec<char> = mv.chars().collect();
        match moves.as_slice() {
            [sc, sr, dc, dr] => {
                let src_pos = to_pos(sc, sr);
                let dest_pos = to_pos(dc, dr);
                self.make_move(Move::new(src_pos, dest_pos, 0));
            },
            _ => () // malformed move
        }
    }

    pub fn get_pseudo_moves(&self) -> Vec<Move> {
        let mut moves: Vec<Move> = Vec::with_capacity(64);

        let white_side = self.to_move == Color::White;
        let (us, opp) = if white_side { (&self.w, &self.b) } else { (&self.b, &self.w) };
        let rank_3    = if white_side { ROW_3 } else { ROW_6 };
        let prom_rank = if white_side { ROW_8 } else { ROW_1 };

        let occ = us.pieces | opp.pieces;

        // Pawn
        let mut pushes = (us.pawn << 8) & !occ;

        let double_pushes = ((pushes & rank_3) << 8) & !occ;

        let left_attacks = (us.pawn << 7) & (opp.pieces | self.en_passant) & !FILE_H;

        let right_attacks = (us.pawn << 9) & (opp.pieces | self.en_passant) & !FILE_A;

        let promotions = pushes & prom_rank; // Get all moves
        // let promotion_captures = (left_attacks | right_attacks) & ROW_8;
        pushes &= !prom_rank; // Remove ROW_8 pushes

        add_moves(&mut moves, pushes, 8);
        add_moves(&mut moves, double_pushes, 16);
        add_moves(&mut moves, left_attacks, 7);
        add_moves(&mut moves, right_attacks, 9);
        add_moves(&mut moves, promotions, 8);

        for_all_pieces(us.queen, &mut moves, &|from, piece| -> u64 {
                (get_line_attacks(occ, file(from), piece) |
                 get_line_attacks(occ, row(from),  piece) |
                 get_line_attacks(occ, diag(from), piece) |
                 get_line_attacks(occ, a_diag(from), piece)) & !us.pieces
            });

        for_all_pieces(us.rook, &mut moves, &|from, piece| -> u64 {
                (get_line_attacks(occ, file(from), piece) |
                 get_line_attacks(occ, row(from), piece)) & !us.pieces
            });

        for_all_pieces(us.bishop, &mut moves, &|from, piece| -> u64 {
                (get_line_attacks(occ, diag(from), piece) |
                 get_line_attacks(occ, a_diag(from), piece)) & !us.pieces
            });

        for_all_pieces(us.knight, &mut moves, &|from, piece| -> u64 {
                KNIGHT_MAP[from as usize] & !us.pieces
            });

        for_all_pieces(us.king, &mut moves, &|from, piece| -> u64 {
                KING_MAP[from as usize] & !us.pieces
            });

        moves
    }

    pub fn get_moves(&mut self) -> Vec<Move> {
        let pseudo_legal_moves = self.get_pseudo_moves();
        let mut legal_moves = Vec::with_capacity(pseudo_legal_moves.len());

        for mv in pseudo_legal_moves.into_iter() {
            println!("\nChecking from {} to {}", mv.from(), mv.to());
            self.make_move(mv);
            println!("{}", self);
            let potent_moves = self.get_pseudo_moves();
            let mut king_is_attacked = false;
            for opp_mv in potent_moves {
                println!("Opponent move from {} to {}", opp_mv.from(), opp_mv.to());
                if opp_mv.to() == self.w.king.trailing_zeros() {
                    println!("Move attacks king");
                    king_is_attacked = true;
                    break;
                }
            }
            if !king_is_attacked {legal_moves.push(mv)};
            self.unmake();
            println!("After unmake\n{}", self);
        }
        legal_moves
    }

    pub fn new(fen_board: &str) -> Board {
        let reversed_rows = fen_board.split('/').rev(); // fen is read from top rank
        let mut sqs = [Square::Empty; 64];

        for (r, row) in reversed_rows.enumerate() {
            let mut offset = 0;
            for (c, ch) in row.chars().enumerate() {
                if !ch.is_numeric() {
                    sqs[r*8 + c+offset] = to_piece(ch);
                } else {
                    offset += (ch as u8 - b'0') as usize;
                }
            }
        }
        let (w, b) = gen_bitboards(&sqs);

        Board { w: w, b: b, sqs: sqs, to_move: Color::White, last_cap: Square::Empty,
            last_move: Move::new(0,0,0), w_castle: true, b_castle: true, en_passant: 0 }
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut characters = Vec::with_capacity(64);

        for (i, sq) in self.sqs.iter().enumerate() {
            characters.push(to_char(sq));
            if (i+1) % 8 == 0 { characters.push('\n') }
        }
        let output = characters.iter().cloned().collect::<String>();
        write!(f, "--------\n{}--------\n\
                  last_cap {:?}\nto_move {:?}\n\
                  wcas {} bcas {}\nen passant {}",
                  output, self.last_cap, self.to_move,
                  self.w_castle, self.b_castle, self.en_passant)
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
