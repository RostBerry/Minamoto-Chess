use minamoto_chess_core::r#move::MoveType as CoreMoveType;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum MoveType {
    Regular,
    PawnDoubleMove,
    PromotionQueen,
    PromotionKnight,
    PromotionRook,
    PromotionBishop,
    CastlingKingSide,
    CastlingQueenSide
}

impl From<CoreMoveType> for MoveType {
    fn from(move_type: CoreMoveType) -> Self {
        match move_type {
            CoreMoveType::Regular => MoveType::Regular,
            CoreMoveType::PawnDoubleMove => MoveType::PawnDoubleMove,
            CoreMoveType::PromotionQueen => MoveType::PromotionQueen,
            CoreMoveType::PromotionKnight => MoveType::PromotionKnight,
            CoreMoveType::PromotionRook => MoveType::PromotionRook,
            CoreMoveType::PromotionBishop => MoveType::PromotionBishop,
            CoreMoveType::CastlingKingSide => MoveType::CastlingKingSide,
            CoreMoveType::CastlingQueenSide => MoveType::CastlingQueenSide,
        }
    }
}

impl From<MoveType> for CoreMoveType {
    fn from(move_type: MoveType) -> Self {
        match move_type {
            MoveType::Regular => CoreMoveType::Regular,
            MoveType::PawnDoubleMove => CoreMoveType::PawnDoubleMove,
            MoveType::PromotionQueen => CoreMoveType::PromotionQueen,
            MoveType::PromotionKnight => CoreMoveType::PromotionKnight,
            MoveType::PromotionRook => CoreMoveType::PromotionRook,
            MoveType::PromotionBishop => CoreMoveType::PromotionBishop,
            MoveType::CastlingKingSide => CoreMoveType::CastlingKingSide,
            MoveType::CastlingQueenSide => CoreMoveType::CastlingQueenSide,
        }
    }
}