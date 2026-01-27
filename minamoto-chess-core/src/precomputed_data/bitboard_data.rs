use super::bitboard_data_gen;

pub struct BitboardData {
    relevant_occupancies: [Box<[u64]>; 2],
    blocker_patterns_data: Box<[u64]>,
    blocker_patterns_offsets: [[usize; 64]; 2],
    blocker_patterns_sizes: [[usize; 64]; 2],
    pseudo_legal_moves_data: Box<[u64]>,
    pseudo_legal_moves_offsets: [[usize; 64]; 2],
    pseudo_legal_moves_sizes: [[usize; 64]; 2],
}

impl BitboardData {
    #[cold]
    pub fn new() -> Box<Self> {
        let mut rook_relevant_occupancies = Vec::with_capacity(64);
        let mut bishop_relevant_occupancies = Vec::with_capacity(64);
        let mut all_blocker_patterns = Vec::new();
        let mut all_pseudo_legal_moves = Vec::new();
        let mut blocker_patterns_offsets = [[0; 64]; 2];
        let mut blocker_patterns_sizes = [[0; 64]; 2];
        let mut pseudo_legal_moves_offsets = [[0; 64]; 2];
        let mut pseudo_legal_moves_sizes = [[0; 64]; 2];

        for square in 0..64 {
            let rook_relevant_occupancy = bitboard_data_gen::generate_relevant_occupancy(
                square, true);
            rook_relevant_occupancies.push(rook_relevant_occupancy);
            let rook_blockers = bitboard_data_gen::generate_blocker_combinations(
                rook_relevant_occupancy);
            let rook_moves = bitboard_data_gen::generate_pseudo_legal_move_combinations(
                square, &rook_blockers, true
            );

            let bishop_relevant_occupancy = bitboard_data_gen::generate_relevant_occupancy(
                square, false);
            bishop_relevant_occupancies.push(bishop_relevant_occupancy);
            let bishop_blockers = bitboard_data_gen::generate_blocker_combinations(
                bishop_relevant_occupancy);
            let bishop_moves = bitboard_data_gen::generate_pseudo_legal_move_combinations(
                square, &bishop_blockers, false
            );

            blocker_patterns_offsets[0][square] = all_blocker_patterns.len();
            blocker_patterns_sizes[0][square] = bishop_blockers.len();
            all_blocker_patterns.extend_from_slice(&bishop_blockers);

            blocker_patterns_offsets[1][square] = all_blocker_patterns.len();
            blocker_patterns_sizes[1][square] = rook_blockers.len();
            all_blocker_patterns.extend_from_slice(&rook_blockers);

            pseudo_legal_moves_offsets[0][square] = all_pseudo_legal_moves.len();
            pseudo_legal_moves_sizes[0][square] = bishop_moves.len();
            all_pseudo_legal_moves.extend_from_slice(&bishop_moves);

            pseudo_legal_moves_offsets[1][square] = all_pseudo_legal_moves.len();
            pseudo_legal_moves_sizes[1][square] = rook_moves.len();
            all_pseudo_legal_moves.extend_from_slice(&rook_moves);
        }

        Box::new(Self {
            relevant_occupancies: [
                bishop_relevant_occupancies.into_boxed_slice(), 
                rook_relevant_occupancies.into_boxed_slice()
            ],
            blocker_patterns_data: all_blocker_patterns.into_boxed_slice(),
            blocker_patterns_offsets,
            blocker_patterns_sizes,
            pseudo_legal_moves_data: all_pseudo_legal_moves.into_boxed_slice(),
            pseudo_legal_moves_offsets,
            pseudo_legal_moves_sizes,
        })
    }

    pub fn get_relevant_occupancy(&self, square: usize, slider_index: usize) -> u64 {
        debug_assert!(slider_index < 2, "Slider index out of bounds");
        debug_assert!(square < 64, "Square index out of bounds");
        unsafe {
            *self.relevant_occupancies.get_unchecked(slider_index).get_unchecked(square)
        }
    }

    pub fn get_blocker_patterns(&self, square: usize, slider_index: usize) -> &[u64] {
        debug_assert!(slider_index < 2, "Slider index out of bounds");
        debug_assert!(square < 64, "Square index out of bounds");
        let offset = unsafe { *self.blocker_patterns_offsets.get_unchecked(slider_index).get_unchecked(square) };
        let size = unsafe { *self.blocker_patterns_sizes.get_unchecked(slider_index).get_unchecked(square) };
        unsafe { 
            std::slice::from_raw_parts(
                self.blocker_patterns_data.as_ptr().add(offset), 
                size
            ) 
        }
    }

    pub fn get_pseudo_legal_moves(&self, square: usize, slider_index: usize) -> &[u64] {
        debug_assert!(slider_index < 2, "Slider index out of bounds");
        debug_assert!(square < 64, "Square index out of bounds");
        let offset = unsafe { *self.pseudo_legal_moves_offsets.get_unchecked(slider_index).get_unchecked(square) };
        let size = unsafe { *self.pseudo_legal_moves_sizes.get_unchecked(slider_index).get_unchecked(square) };
        unsafe { 
            std::slice::from_raw_parts(
                self.pseudo_legal_moves_data.as_ptr().add(offset), 
                size
            ) 
        }
    }
}
