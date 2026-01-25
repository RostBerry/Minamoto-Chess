use rand::{rng, rngs::ThreadRng, RngCore};

use super::{magic_bitboard_data, magic_score::MagicScore, square_magic::SquareMagic};

fn generate_random_number(rng: &mut ThreadRng) -> u64 {
    let a = rng.next_u64() & 0xFFFF;
    let b = rng.next_u64() & 0xFFFF;
    let c = rng.next_u64() & 0xFFFF;
    let d = rng.next_u64() & 0xFFFF;
    a | (b << 16) | (c << 32) | (d << 48)
}

fn generate_magic_candidate(rng: &mut ThreadRng) -> u64 {
    generate_random_number(rng) & generate_random_number(rng) & generate_random_number(rng)
}

pub enum MagicValidationResult {
    Valid(MagicScore),
    Invalid
}

pub fn validate_magic_number(magic: u64, shift: usize, square: usize, slider_index: usize) -> MagicValidationResult {
    let blocker_patterns = super::BITBOARD_DATA.get_blocker_patterns(square, slider_index);
    let blocker_pattern_count = blocker_patterns.len();

    if blocker_pattern_count != magic_bitboard_data::get_relevant_occupancy_cardinality(
        super::BITBOARD_DATA.get_relevant_occupancy(square, slider_index)
    ) {
        panic!("Incorrect blocker pattern generation");
    }

    let mut used_indices = Vec::with_capacity(blocker_pattern_count);
    used_indices.resize(blocker_pattern_count, false);
    let mut used_patterns = Vec::with_capacity(blocker_pattern_count);
    used_patterns.resize(blocker_pattern_count, 0);

    let mut max_index = 0;

    for pattern_index in 0..blocker_pattern_count {
        let pattern = blocker_patterns[pattern_index];
        let pseudo_legal_move_pattern = super::BITBOARD_DATA
            .get_pseudo_legal_moves(square, slider_index)[pattern_index];
        let lookup_index = magic_bitboard_data::generate_lookup_table_index(
            magic, 
            shift, 
            pattern, 
            super::BITBOARD_DATA.get_relevant_occupancy(square, slider_index),
        );

        if lookup_index >= blocker_pattern_count {
            return MagicValidationResult::Invalid;
        }

        if used_indices[lookup_index] {
            if used_patterns[lookup_index] != pseudo_legal_move_pattern {
                return MagicValidationResult::Invalid;
            }
        } 
        used_indices[lookup_index] = true;
        used_patterns[lookup_index] = pseudo_legal_move_pattern;

        max_index = max_index.max(lookup_index);
    }

    MagicValidationResult::Valid(MagicScore::from_validated(square, max_index, slider_index))
}

pub fn generate_magic_number(square: usize, slider_index: usize) -> SquareMagic {
    let mut rng = rng();

    let shift = magic_bitboard_data::get_shift(square, slider_index);
    
    loop {
        let candidate = generate_magic_candidate(&mut rng);
            match validate_magic_number(candidate, shift, square, slider_index) {
                MagicValidationResult::Valid(score) => {
                    return SquareMagic::with_score(square, candidate, shift, score);
                },
                MagicValidationResult::Invalid => continue
            }
    }
}