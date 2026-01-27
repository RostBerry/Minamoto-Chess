use minamoto_chess_core::{board::Board, piece};

use crate::board_representation;

pub trait BoardExtensions {
    fn load_position(&mut self, position: &str);
}

impl BoardExtensions for Board {
    /// Loads position from FEN string (only piece placement part)
    fn load_position(&mut self, fen_pos: &str) {
        let rows: Vec<&str> = fen_pos.split("/").collect();

        for y in 0..8usize {
            let mut x = 7i8;

            for sym in rows[7 - y].chars() {
                if x < 0 {
                    continue;
                }

                if sym.is_digit(10) {
                    x -= sym.to_digit(10).expect("FEN loading") as i8;
                    continue;
                }

                let (color, piece_type) = board_representation::get_piece_from_fen(&sym);
                let square = x as usize + y * 8;
                if *piece_type != piece::NONE {
                    self.create_piece_public(square, *color, *piece_type);

                }
                x -= 1;
            }
        }
    }
}