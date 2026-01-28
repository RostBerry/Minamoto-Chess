use minamoto_chess_core::move_generation::attack_calculator::AttackCalculator;
use serde::{Deserialize, Serialize};
use tsify::Tsify;

fn attacked_squares_bb_to_vec(attacked_squares_bb: u64) -> Vec<usize> {
    let mut squares = Vec::new();
    for square in 0..64 {
        if (attacked_squares_bb >> square) & 1 == 1 {
            squares.push(square);
        }
    }
    squares
}

#[derive(Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi)]
pub struct AttackInfo {
    attacked_squares: Vec<usize>,
}

impl AttackInfo {
    pub fn new(attacked_squares_bb: u64) -> Self {
        let attacked_squares = attacked_squares_bb_to_vec(attacked_squares_bb);

        Self {
            attacked_squares,
        }
    }

    pub fn from_attack_calculator(attack_calculator: &AttackCalculator) -> Self {
        Self::new(attack_calculator.squares_in_attack_bb[0])
    }

    pub fn get_attacked_squares(&self) -> &Vec<usize> {
        &self.attacked_squares
    }
}