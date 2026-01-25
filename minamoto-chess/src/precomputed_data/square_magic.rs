use super::{magic_bitboard_gen, magic_bitboard_data::{get_magic_number, get_shift}, magic_score::MagicScore};

/// Contains essential information for the magics
#[derive(Clone)]
pub struct SquareMagic {
    square: usize,
    magic: u64,
    shift: usize,
    score: MagicScore,
}

impl SquareMagic {
    /// Creates *SquareMagic* from the magic number saved on that square in the constant
    pub fn from_existent(square: usize, slider_index: usize) -> Self {
        let magic = get_magic_number(square, slider_index);
        let shift = get_shift(square, slider_index);
        let score = match magic_bitboard_gen::validate_magic_number(magic, shift, square, slider_index) {
            magic_bitboard_gen::MagicValidationResult::Valid(score) => score,
            magic_bitboard_gen::MagicValidationResult::Invalid => panic!("Magic number is invalid")
        };
        Self {
            square,
            magic,
            shift,
            score,
        }
    }
    /// Creates *SquareMagic* from the provided magic number
    pub fn new(square: usize, slider_index: usize, magic: u64, shift: usize) -> Self {
        let score = match magic_bitboard_gen::validate_magic_number(magic, shift, square, slider_index) {
            magic_bitboard_gen::MagicValidationResult::Valid(score) => score,
            magic_bitboard_gen::MagicValidationResult::Invalid => panic!("Magic number is invalid")
        };
        Self {
            square,
            magic,
            shift,
            score
        }
    }

    /// Creates *SquareMagic* from the provided magic number and score
    pub fn with_score(square: usize, magic: u64, shift: usize, score: MagicScore) -> Self {
        Self {
            square,
            magic,
            shift,
            score
        }
    }

    pub fn get_square(&self) -> usize {
        self.square
    }

    pub fn get_magic(&self) -> u64 {
        self.magic
    }

    pub fn get_shift(&self) -> usize {
        self.shift
    }

    pub fn get_score(&self) -> &MagicScore {
        &self.score
    }
}