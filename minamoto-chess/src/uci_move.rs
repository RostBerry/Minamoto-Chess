use std::fmt::Display;

use minamoto_chess_core::{r#move::{Move}, piece};
use serde::{Deserialize, Serialize};
use tsify::Tsify;

use crate::{board_representation::{get_piece_from_fen, get_square_from_name, get_square_name, piece_to_fen_sym}};

#[derive(Tsify, Serialize, Deserialize, PartialEq)]
#[tsify(into_wasm_abi)]
pub enum PromotionType {
    Queen,
    Knight,
    Rook,
    Bishop
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct UciMove {
    pub start_square: usize,
    pub target_square: usize,
    pub promotion: Option<PromotionType>
}

pub enum UciMoveCreationResult {
    Success(UciMove),
    Failure
}

impl UciMove {
    pub fn new(start_square: usize, target_square: usize, promotion: Option<PromotionType>) -> Self {
        Self {
            start_square,
            target_square,
            promotion
        }
    }

    /// Accepts a UCI move (e.g e2e4, e7e8q) as a string and returns a UciMove if the move is valid
    pub fn from_str(uci: &str) -> UciMoveCreationResult {
        if uci.len() >= 4 && uci.len() <= 5 {
            let start_square = get_square_from_name(&uci[0..2]);
            let target_square = get_square_from_name(&uci[2..4]);
            let mut promotion = None;

            if uci.len() == 5 {
                promotion = match get_piece_from_fen(
                    &uci.chars().nth(4).expect("Something with move from uci")
                ).1 {
                    piece::QUEEN => Some(PromotionType::Queen),
                    piece::KNIGHT => Some(PromotionType::Knight),
                    piece::ROOK => Some(PromotionType::Rook),
                    piece::BISHOP => Some(PromotionType::Bishop),
                    _ => None
                }
            }
            return UciMoveCreationResult::Success(UciMove::new(start_square, target_square, promotion));
        }
        UciMoveCreationResult::Failure
    }

    pub fn from_move(mov: Move) -> Self {
        Self {
            start_square: mov.start_square,
            target_square: mov.target_square,
            promotion: match mov.move_type {
                minamoto_chess_core::r#move::MoveType::PromotionQueen => Some(PromotionType::Queen),
                minamoto_chess_core::r#move::MoveType::PromotionKnight => Some(PromotionType::Knight),
                minamoto_chess_core::r#move::MoveType::PromotionRook => Some(PromotionType::Rook),
                minamoto_chess_core::r#move::MoveType::PromotionBishop => Some(PromotionType::Bishop),
                _ => None
            }
        }
    }

    pub fn is_promotion(&self) -> bool {
        match &self.promotion {
            Some(_) => true,
            None => false
        }
    }
}

impl Display for UciMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "{}{}{}", 
            get_square_name(self.start_square), 
            get_square_name(self.target_square), 
            match &self.promotion {
                Some(PromotionType::Queen) => piece_to_fen_sym(piece::BLACK, piece::QUEEN).to_string(),
                Some(PromotionType::Knight) => piece_to_fen_sym(piece::BLACK, piece::KNIGHT).to_string(),
                Some(PromotionType::Rook) => piece_to_fen_sym(piece::BLACK, piece::ROOK).to_string(),
                Some(PromotionType::Bishop) => piece_to_fen_sym(piece::BLACK, piece::BISHOP).to_string(),
                None => String::new()
            }
        )
    }
}