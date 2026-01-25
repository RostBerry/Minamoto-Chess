use minamoto_chess::board::Board;
use wasm_bindgen::prelude::*;

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