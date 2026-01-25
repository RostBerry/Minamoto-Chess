use std::mem::MaybeUninit;

use crate::bitboards;

use super::{SQUARE_DATA, square_data::get_move_offset};

pub fn generate_relevant_occupancy(square: usize, is_rook: bool) -> u64 {
    let start_direction_index = if is_rook {0} else {4};
    let end_direction_index = if is_rook {4} else {8};

    let mut relevant_occupancy = 0;

    for direction_index in start_direction_index..end_direction_index {
        let squares_to_edge = SQUARE_DATA
            .get_squares_to_edge(square, direction_index)
            as isize;

        if squares_to_edge > 1 {
            for distance in 1..squares_to_edge {
                let offset = get_move_offset(direction_index) * distance;
                let target_square = (square as isize + offset) as usize;

                relevant_occupancy |= bitboards::get_bit_from_square(target_square);
            }
        }
    }

    relevant_occupancy
}

pub fn generate_blocker_combinations(relevant_occ: u64) -> Box<[u64]> {
    let num_bits = relevant_occ.count_ones();
    let max_combinations = 1usize << num_bits;

    let mut combinations: Box<[MaybeUninit<u64>]> = {
        let mut vec = Vec::with_capacity(max_combinations);
        unsafe {
            vec.set_len(max_combinations);
            vec.into_boxed_slice().try_into().unwrap_unchecked()
        }
    };
    
    // Convert relevant occupancy to array of bit positions
    let mut bit_positions = [0u8; 64];
    let mut bit_count = 0;
    for i in 0..64 {
        if (relevant_occ >> i) & 1 == 1 {
            bit_positions[bit_count] = i as u8;
            bit_count += 1;
        }
    }
    
    // Generate all possible combinations
    for i in 0..max_combinations {
        let mut blocker = 0u64;
        for j in 0..bit_count {
            if (i >> j) & 1 == 1 {
                blocker |= 1u64 << bit_positions[j];
            }
        }
        combinations[i].write(blocker);
    }
    
    unsafe {
        std::mem::transmute::<Box<[MaybeUninit<u64>]>, Box<[u64]>>(combinations)
    }
}

pub fn generate_pseudo_legal_move_combinations(square: usize, blocker_combinations: &Box<[u64]>, is_rook: bool) -> Box<[u64]> {
    let mut combinations: Box<[MaybeUninit<u64>]> = {
        let mut vec = Vec::with_capacity(blocker_combinations.len());
        unsafe {
            vec.set_len(blocker_combinations.len());
            vec.into_boxed_slice().try_into().unwrap_unchecked()
        }
    };

    let start_direction_index = if is_rook {0} else {4};
    let end_direction_index = if is_rook {4} else {8};


    for (i, pattern) in blocker_combinations.into_iter().enumerate() {
        let mut pseudo_legal_moves_bb = 0;

        for direction_index in start_direction_index..end_direction_index {
            let squares_to_edge = SQUARE_DATA
                .get_squares_to_edge(square, direction_index)
                as isize;
    
            if squares_to_edge > 0 {
                for distance in 1..=squares_to_edge {
                    let offset = get_move_offset(direction_index) * distance;
                    let target_square = (square as isize + offset) as usize;
                    let target_square_bb = bitboards::get_bit_from_square(target_square);
    
                    pseudo_legal_moves_bb |= target_square_bb;
                    
                    if target_square_bb & pattern != 0 {
                        break;
                    }
                }
            }
        }

        combinations[i].write(pseudo_legal_moves_bb);
    }

    unsafe {
        std::mem::transmute::<Box<[MaybeUninit<u64>]>, Box<[u64]>>(combinations)
    }
}