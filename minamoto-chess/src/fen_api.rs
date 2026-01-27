use minamoto_chess_core::{board::Board, castling, piece::{self, BLACK, WHITE}, zobrist};

use crate::{board_extensions::BoardExtensions, board_representation};

pub trait FenApi {
    fn from_fen(fen: &str) -> Self;
    fn to_fen(&self) -> String;
}

impl FenApi for Board {    
    /// Creates board from a position in the provided FEN string
    fn from_fen(fen_string: &str) -> Self {
        let fen_data: Vec<&str> = fen_string.split(" ").collect();

        let mut board = Self::empty();
        board.load_position(fen_data[0]);

        if fen_data.len() > 1 && fen_data[1] == "b" {
            board.switch_color();
        }

        if fen_data.len() > 2 {
            for i in 0..2 {
                let can_short = fen_data[2].contains(if i == 0 {'K'} else {'k'});
                let can_long = fen_data[2].contains(if i == 0 {'Q'} else {'q'});

                *board.get_castling_state_mut(i) = castling::BOTH_SIDES;

                if !can_short {
                    castling::annul_king_side(board.get_castling_state_mut(i));
                }
                if !can_long {
                    castling::annul_queen_side(board.get_castling_state_mut(i));
                }
            }
        }

        // Set en passant state if available
        if fen_data.len() > 3 && fen_data[3] != "-" {
            let square = board_representation::get_square_from_name(fen_data[3]);
            let pawn_offset = if board.get_current_color() == WHITE { 8 } else { -8i32 };
            let capture_square = square;
            let pawn_square = (square as i32 + pawn_offset) as usize;
            board.update_en_passant_state_public(true, pawn_square, capture_square);
        }

        // Set halfmoves for 50-move rule
        if fen_data.len() > 4 {
            if let Ok(halfmoves) = fen_data[4].parse::<u8>() {
                *board.rule50_count_mut() = halfmoves;
            }
        }

        // Set move counter
        if fen_data.len() > 5 {
            if let Ok(moves) = fen_data[5].parse::<u16>() {
                *board.get_move_counter_mut() = moves;
            } else {
                *board.get_move_counter_mut() = 1;
            }
        } else {
            *board.get_move_counter_mut() = 1;
        }
        
        // Calculate initial hash after position is set up
        *board.get_zobrist_hash_mut() = zobrist::calculate_hash(&board);
        let zobrist_hash = board.get_zobrist_hash();
        (*board.get_position_history_mut()).insert(zobrist_hash, 1);

        board
    }

    /// Returns the FEN string of the current position
    fn to_fen(&self) -> String {
        let mut fen_string = String::new();

        for y in (0..8).rev() {
            let mut empty_squares = 0;

            for x in (0..8).rev() {
                let square = x + y * 8;
                let (color, piece_type) = self.get_piece_on_square(square);

                if piece_type == piece::NONE {
                    empty_squares += 1;
                } else {
                    if empty_squares > 0 {
                        fen_string.push_str(&empty_squares.to_string());
                        empty_squares = 0;
                    }

                    fen_string.push(board_representation::piece_to_fen_sym(color, piece_type));
                }
            }

            if empty_squares > 0 {
                fen_string.push_str(&empty_squares.to_string());
            }

            if y > 0 {
                fen_string.push('/');
            }
        }

        fen_string.push(' ');

        fen_string.push(if self.is_white_to_move() {'w'} else {'b'});
        fen_string.push(' ');

        let mut castling_string = String::new();
        if castling::can_king_side(self.get_castling_state(WHITE)) {
            castling_string.push('K');
        }
        if castling::can_queen_side(self.get_castling_state(WHITE)) {
            castling_string.push('Q');
        }
        if castling::can_king_side(self.get_castling_state(BLACK)) {
            castling_string.push('k');
        }
        if castling::can_queen_side(self.get_castling_state(BLACK)) {
            castling_string.push('q');
        }
        if castling_string.is_empty() {
            castling_string.push('-');
        }
        fen_string.push_str(&castling_string);

        fen_string.push(' ');

        if self.is_en_passant_possible() {
            fen_string.push_str(
                &board_representation::get_square_name(
                    self.en_passant_capture_square()
                )
            );
        } else {
            fen_string.push('-');
        }

        // Add halfmove clock (50-move rule counter)
        fen_string.push(' ');
        fen_string.push_str(&self.rule50_count().to_string());

        // Add fullmove number
        fen_string.push(' ');
        fen_string.push_str(&self.get_move_counter().to_string());
        fen_string
    }
}