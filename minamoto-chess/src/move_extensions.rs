use minamoto_chess_core::{bitboards, board::{self, Board}, r#move::{Move, MoveType}, piece};

use crate::{board_representation::{get_square_name, piece_to_fen_sym}, uci_move::{PromotionType, UciMove}};

pub fn print(moves: &[Move]) {
    println!("All moves({}):", moves.len());
    for mov in moves {
        print!("{} ", move_to_string(mov));
    }
    println!();
}

pub fn move_to_string(mov: &Move) -> String {
    format!(
        "{}{}{}", 
        get_square_name(mov.start_square), 
        get_square_name(mov.target_square), 
        match MoveType::from(mov.move_type) {
            MoveType::PromotionQueen => piece_to_fen_sym(piece::BLACK, piece::QUEEN).to_string(),
            MoveType::PromotionKnight => piece_to_fen_sym(piece::BLACK, piece::KNIGHT).to_string(),
            MoveType::PromotionRook => piece_to_fen_sym(piece::BLACK, piece::ROOK).to_string(),
            MoveType::PromotionBishop => piece_to_fen_sym(piece::BLACK, piece::BISHOP).to_string(),
            _ => String::new()
        }
    )
}

pub trait MoveExtensions {
    fn from_uci(mov: UciMove, board: &Board) -> Self;
}

impl MoveExtensions for Move {
    fn from_uci(mov: UciMove, board: &Board) -> Self {
        let start_square = mov.start_square;
        let target_square = mov.target_square;
        let target_square_bb = bitboards::get_bit_from_square(target_square);
        let (color, piece_type) = board.get_piece_on_square(start_square);
        let mut capture_square = target_square;

        if piece_type == piece::PAWN && board.is_en_passant_possible() && target_square_bb == bitboards::get_bit_from_square(board.en_passant_capture_square()) {
            capture_square = board.en_passant_pawn_square();
        }

        let mut move_type = MoveType::Regular;

        if let Some(promotion_type) = mov.promotion {
            move_type = match promotion_type {
                PromotionType::Queen => MoveType::PromotionQueen,
                PromotionType::Knight => MoveType::PromotionKnight,
                PromotionType::Rook => MoveType::PromotionRook,
                PromotionType::Bishop => MoveType::PromotionBishop,
            };
        }

        if piece_type == piece::KING && start_square == board::get_king_start_square(color) {
            if target_square == board::get_king_side_square(color) {
                move_type = MoveType::CastlingKingSide;
            } else if target_square == board::get_queen_side_square(color) {
                move_type = MoveType::CastlingQueenSide;
            }
        }

        if piece_type == piece::PAWN && ((target_square as i32) - (start_square as i32)).abs() == 16 {
            move_type = MoveType::PawnDoubleMove;
        }

        Self {
            start_square,
            target_square,
            capture_square,
            move_type: move_type.into()
        }
    }
}