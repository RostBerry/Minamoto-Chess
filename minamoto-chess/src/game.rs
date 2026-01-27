use minamoto_chess_core::{board::Board, r#move::Move, move_generation::attack_calculator::AttackCalculator, piece};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum GameMode {
    PvP,
    PvB,
    BvB
}

#[derive(Serialize, Deserialize)]
pub enum GameState {
    InProgress,
    WhiteWon,
    BlackWon,
    Draw
}

impl GameState {
    pub fn from_current_state(board: &Board, legal_moves: &[Move], attack_calc: &AttackCalculator) -> GameState {
        if legal_moves.is_empty() {
            if attack_calc.in_check() {
                match board.get_current_color() {
                    piece::WHITE => return GameState::BlackWon,
                    piece::BLACK => return GameState::WhiteWon,
                    _ => unreachable!(),
                }
            } else {
                return GameState::Draw;
            }
        }

        if board.is_draw_by_repetition() {
            return GameState::Draw;
        }

        if board.is_draw_by_50_moves_rule() {
            return GameState::Draw;
        }

        if board.is_draw_by_material() {
            return GameState::Draw;
        }

        GameState::InProgress
    }
}