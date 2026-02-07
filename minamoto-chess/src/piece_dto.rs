use serde::{Deserialize, Serialize};
use tsify::Tsify;

#[derive(Tsify, Serialize, Deserialize, Debug, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum PieceType {
    King,
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen
}

impl PieceType {
    pub fn from_num(num: usize) -> Option<Self> {
        match num {
            1 => Some(PieceType::King),
            2 => Some(PieceType::Pawn),
            3 => Some(PieceType::Knight),
            4 => Some(PieceType::Bishop),
            5 => Some(PieceType::Rook),
            6 => Some(PieceType::Queen),
            _ => None,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Debug, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum PieceColor {
    White,
    Black
}

impl PieceColor {
    pub fn from_num(num: usize) -> Option<Self> {
        match num {
            0 => Some(PieceColor::White),
            1 => Some(PieceColor::Black),
            _ => None,
        }
    }
}

#[derive(Tsify, Serialize, Deserialize, Debug, Clone)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Piece {
    pub piece_type: PieceType,
    pub color: PieceColor
}

#[derive(Tsify, Serialize, Deserialize, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct PiecePlacement {
    pub piece: Piece,
    pub square: usize
}