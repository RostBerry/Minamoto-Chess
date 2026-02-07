use minamoto_chess_core::{board::Board, piece};

use crate::board_representation;

pub trait BoardExtensions {
    fn load_position(&mut self, position: &str);
    fn count_material(&self) -> i8;
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

    /// Returns positive if white has more material, negative if black has more, 0 if equal.
    /// 
    /// Uses standard piece values: Pawn=1, Knight=3, Bishop=3, Rook=5, Queen=9
    /// King is not counted.
    #[inline]
    fn count_material(&self) -> i8 {
        const PAWN_VALUE: i8 = 1;
        const KNIGHT_VALUE: i8 = 3;
        const BISHOP_VALUE: i8 = 3;
        const ROOK_VALUE: i8 = 5;
        const QUEEN_VALUE: i8 = 9;

        let w_pawns   = self.get_piece_bitboard(piece::WHITE, piece::PAWN).count_ones() as i8;
        let w_knights = self.get_piece_bitboard(piece::WHITE, piece::KNIGHT).count_ones() as i8;
        let w_bishops = self.get_piece_bitboard(piece::WHITE, piece::BISHOP).count_ones() as i8;
        let w_rooks   = self.get_piece_bitboard(piece::WHITE, piece::ROOK).count_ones() as i8;
        let w_queens  = self.get_piece_bitboard(piece::WHITE, piece::QUEEN).count_ones() as i8;

        let b_pawns   = self.get_piece_bitboard(piece::BLACK, piece::PAWN).count_ones() as i8;
        let b_knights = self.get_piece_bitboard(piece::BLACK, piece::KNIGHT).count_ones() as i8;
        let b_bishops = self.get_piece_bitboard(piece::BLACK, piece::BISHOP).count_ones() as i8;
        let b_rooks   = self.get_piece_bitboard(piece::BLACK, piece::ROOK).count_ones() as i8;
        let b_queens  = self.get_piece_bitboard(piece::BLACK, piece::QUEEN).count_ones() as i8;

        let white_material = w_pawns * PAWN_VALUE
                           + w_knights * KNIGHT_VALUE
                           + w_bishops * BISHOP_VALUE
                           + w_rooks * ROOK_VALUE
                           + w_queens * QUEEN_VALUE;

        let black_material = b_pawns * PAWN_VALUE
                           + b_knights * KNIGHT_VALUE
                           + b_bishops * BISHOP_VALUE
                           + b_rooks * ROOK_VALUE
                           + b_queens * QUEEN_VALUE;

        white_material - black_material
    }
}