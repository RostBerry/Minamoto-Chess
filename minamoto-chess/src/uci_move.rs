use std::fmt::Display;

use minamoto_chess_core::{r#move::{Move}, piece};
use serde::{Deserialize, Serialize};

use crate::{board_representation::{get_piece_from_fen, get_square_from_name, get_square_name, piece_to_fen_sym}, move_type_wrapper::MoveType};

#[derive(Serialize, Deserialize)]
pub struct UciMove {
    pub move_type: MoveType,
    pub start_square: usize,
    pub target_square: usize
}

pub enum UciMoveCreationResult {
    Success(UciMove),
    Failure
}

impl UciMove {
    pub fn new(move_type: MoveType, start_square: usize, target_square: usize) -> Self {
        Self {
            move_type,
            start_square,
            target_square
        }
    }

    /// Accepts a UCI move (e.g e2e4, e7e8q) as a string and returns a UciMove if the move is valid
    pub fn from_str(uci: &str) -> UciMoveCreationResult {
        if uci.len() >= 4 && uci.len() <= 5 {
            let start_square = get_square_from_name(&uci[0..2]);
            let target_square = get_square_from_name(&uci[2..4]);
            let mut move_type = MoveType::Regular;
            if uci.len() == 5 {
                let promotion = get_piece_from_fen(
                    &uci.chars().nth(4).expect("Something with move from uci")
                ).1;

                move_type = match promotion {
                    piece::QUEEN => MoveType::PromotionQueen,
                    piece::KNIGHT => MoveType::PromotionKnight,
                    piece::ROOK => MoveType::PromotionRook,
                    piece::BISHOP => MoveType::PromotionBishop,
                    _ => MoveType::Regular
                };
            }
            return UciMoveCreationResult::Success(UciMove::new(move_type, start_square, target_square));
        }
        UciMoveCreationResult::Failure
    }

    pub fn from_move(mov: Move) -> Self {
        Self {
            move_type: MoveType::from(mov.move_type),
            start_square: mov.start_square,
            target_square: mov.target_square
        }
    }

    pub fn is_promotion(&self) -> bool {
        match &self.move_type {
            MoveType::PromotionQueen | MoveType::PromotionKnight | MoveType::PromotionRook | MoveType::PromotionBishop => true,
            _ => false
        }
    }
}

impl Display for UciMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "{}{}{}", 
            get_square_name(self.start_square), 
            get_square_name(self.target_square), 
            match self.move_type {
                MoveType::PromotionQueen => piece_to_fen_sym(piece::BLACK, piece::QUEEN).to_string(),
                MoveType::PromotionKnight => piece_to_fen_sym(piece::BLACK, piece::KNIGHT).to_string(),
                MoveType::PromotionRook => piece_to_fen_sym(piece::BLACK, piece::ROOK).to_string(),
                MoveType::PromotionBishop => piece_to_fen_sym(piece::BLACK, piece::BISHOP).to_string(),
                _ => String::new()
            }
        )
    }
}