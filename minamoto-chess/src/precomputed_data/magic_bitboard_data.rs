use super::magic_loader;

/// Returns the number of possible blocker patterns for the provided square
pub const fn get_relevant_occupancy_cardinality(relevant_occ: u64) -> usize {
    1 << relevant_occ.count_ones()
}

/// Returns the number of all distinct attack sets for the provided square for the rook
pub fn get_rook_distinct_attack_set_count(square: usize) -> usize {
    let mut result: usize = 1;
    let start_index: usize = 0;
    let end_index: usize = 4;
    for direction_index in start_index..end_index {
        let squares = super::SQUARE_DATA.get_squares_to_edge(square, direction_index);
        result *= if squares != 0 {squares} else {1};
    }

    result
}
/// Returns the number of all distinct attack sets for the provided square for the bishop
pub fn get_bishop_distinct_attack_set_count(square: usize) -> usize {
    let mut result: usize = 1;
    let start_index: usize = 4;
    let end_index: usize = 8;
    for direction_index in start_index..end_index {
        let squares = super::SQUARE_DATA.get_squares_to_edge(square, direction_index);
        result *= if squares != 0 {squares} else {1};
    }

    result
}
/// Returns the number of all distinct attack sets for the provided square
pub fn get_distinct_attack_set_count(slider_index: usize, square: usize) -> usize {
    match slider_index {
        0 => get_bishop_distinct_attack_set_count(square),
        1 => get_rook_distinct_attack_set_count(square),
        _ => panic!("Invalid slider index")
    }
}


/// Returns the minimal possible size for the provided square in the lookup table in bytes
pub fn get_min_lookup_square_size(slider_index: usize, square: usize) -> usize {
    (get_min_bits(get_distinct_attack_set_count(slider_index, square)) + 7) / 8 // Round up to nearest byte
}

/// Returns the minimal number of bits to store all indices of the lookup table for the provided number
pub fn get_min_bits(n: usize) -> usize {
    (usize::BITS - (n - 1).leading_zeros()) as usize
}

/// Returns the minimal number of bits to store all indices of the lookup table
pub fn get_shift(square: usize, slider_index: usize) -> usize {
    debug_assert!(slider_index < 2, "Slider index out of bounds");
    debug_assert!(square < 64, "Square index out of bounds");
    unsafe {
        *magic_loader::SHIFTS.get_unchecked(slider_index).get_unchecked(square)
    }
}

/// Returns the magic number for the provided square
pub fn get_magic_number(square: usize, slider_index: usize) -> u64 {
    debug_assert!(slider_index < 2, "Slider index out of bounds");
    debug_assert!(square < 64, "Square index out of bounds");
    unsafe {
        *magic_loader::MAGIC_NUMBERS.get_unchecked(slider_index).get_unchecked(square)
    }
}

/// Returns the magic bitboard lookup table index for the provided square and blocker pattern
pub fn generate_lookup_table_index(magic: u64, bits_to_shift: usize, blockers: u64, relevant_occ: u64) -> usize {
    debug_assert!(bits_to_shift < 64, "bits_to_shift must be less than 64");
    
    ((blockers & relevant_occ).wrapping_mul(magic) >> (64 - bits_to_shift)) as usize
}

