use minamoto_chess_core::board::Board;
use wasm_bindgen::prelude::*;

use crate::fen_api::FenApi;

pub mod fen_api;
pub mod perft;
pub mod board_representation;
pub mod attack_calc_extensions;
pub mod board_extenstions;
pub mod move_extensions;
pub mod uci_move;
pub mod config;
pub mod magic_bitboard_gen;

#[wasm_bindgen]
pub struct WasmBoard {
    board: Board,
}

#[wasm_bindgen]
impl WasmBoard {
    #[wasm_bindgen(constructor)]
    pub fn new(fen: &str) -> WasmBoard {
        WasmBoard { board: Board::from_fen(fen) }
    }

    #[wasm_bindgen]
    pub fn to_fen(&self) -> String {
        self.board.to_fen()
    }
}