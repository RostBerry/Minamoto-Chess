use minamoto_chess_core::{board::Board, r#move::Move, move_generation::{attack_calculator::AttackCalculator, move_gen::{self}}};
use wasm_bindgen::prelude::*;

use crate::{attack_info::AttackInfo, fen_api::FenApi, game::GameState, move_extensions::MoveExtensions, uci_move::{UciMove, UciMoveCreationResult}};

pub mod fen_api;
pub mod perft;
pub mod board_representation;
pub mod attack_calc_extensions;
pub mod board_extensions;
pub mod move_extensions;
pub mod uci_move;
pub mod config;
pub mod magic;
pub mod move_type_wrapper;
pub mod attack_info;
pub mod game;

#[wasm_bindgen]
pub struct Game {
    board: Board,
}

#[wasm_bindgen]
impl Game {
    #[wasm_bindgen(constructor)]
    pub fn from_fen(fen: &str) -> Game {
        Game { board: Board::from_fen(fen) }
    }

    #[wasm_bindgen]
    pub fn to_fen(&self) -> String {
        self.board.to_fen()
    }

    #[wasm_bindgen]
    pub fn get_attack_info(&self) -> JsValue {
        let attack_calc = AttackCalculator::new(&self.board);
        let attack_info = AttackInfo::from_attack_calculator(&attack_calc);

        serde_wasm_bindgen::to_value(&attack_info)
            .unwrap()
    }

    #[wasm_bindgen]
    pub fn get_all_legal_moves(&self) -> JsValue {
        let attack_calc = AttackCalculator::new(&self.board);

        let mut moves: Vec<Move> = move_gen::create_empty_move_buffer();
        move_gen::generate_moves(&mut moves, &self.board, &attack_calc);

        let uci_moves: Vec<UciMove> = moves.into_iter()
            .map(|m| UciMove::from_move(m))
            .collect();

        serde_wasm_bindgen::to_value(&uci_moves)
            .unwrap()
    }

    #[wasm_bindgen]
    pub fn make_move(&mut self, uci_move_str: &str) -> Result<JsValue, String> {
        match UciMove::from_str(uci_move_str) {
            UciMoveCreationResult::Success(uci_move) => {
                let mov = Move::from_uci(uci_move, &self.board);
                let _move_record = self.board.make_move(mov);

                let attack_calc = AttackCalculator::new(&self.board);
                let mut moves: Vec<Move> = move_gen::create_empty_move_buffer();
                move_gen::generate_moves(&mut moves, &self.board, &attack_calc);

                let game_state = GameState::from_current_state(&self.board, &moves, &attack_calc);
                Ok(serde_wasm_bindgen::to_value(&game_state).unwrap())
            },
            UciMoveCreationResult::Failure => Err("Invalid move".to_string()),
        }
    }
}