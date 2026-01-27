use std::mem::MaybeUninit;

use super::{magic_bitboards, magic_lookup_moves::MagicLookupMoves, square_magic::SquareMagic};

pub struct MagicLookupTable {
    pseudo_legal_moves: [MagicLookupMoves; 64]
}

impl MagicLookupTable {
    #[cold]
    pub fn new(slider_index: usize) -> Box<Self> {
        let mut lookup_table: [MaybeUninit<MagicLookupMoves>; 64] = 
            unsafe { MaybeUninit::uninit().assume_init() };

        for square in 0..64 {
            let blocker_patterns = super::BITBOARD_DATA.get_blocker_patterns(square, slider_index);
            let blocker_pattern_count = blocker_patterns.len();

            let magic = SquareMagic::from_existent(square, slider_index);

            let pseudo_legal_move_pattern_count = magic.get_score().get_max_index() + 1;

            let mut pseudo_legal_move_patterns = vec![0; pseudo_legal_move_pattern_count];

            for pattern_index in 0..blocker_pattern_count {
                let pattern = blocker_patterns[pattern_index];
                let pseudo_legal_move_pattern = super::BITBOARD_DATA
                    .get_pseudo_legal_moves(square, slider_index)[pattern_index];

                let lookup_index = magic_bitboards::generate_lookup_table_index(magic.get_magic(), 
                    magic.get_shift(), 
                    pattern, 
                    super::BITBOARD_DATA.get_relevant_occupancy(square, slider_index),
                );

                pseudo_legal_move_patterns[lookup_index] = pseudo_legal_move_pattern;
            }

            let magic_lookup_move = MagicLookupMoves::new(
                pseudo_legal_move_patterns, 
                super::BITBOARD_DATA.get_relevant_occupancy(square, slider_index),
                magic
            );

            lookup_table[square].write(magic_lookup_move);
        }

        let initialized_lookup_table = unsafe {
            std::mem::transmute::<_, [MagicLookupMoves; 64]>(lookup_table)
        };

        Box::new(Self {
            pseudo_legal_moves: initialized_lookup_table
        })
    }

    pub fn get_pseudo_legal_move_bb(&self, square: usize, blockers: u64) -> u64 {
        debug_assert!(square < 64, "Square index out of bounds");
        unsafe {
            self.pseudo_legal_moves.get_unchecked(square).get_pseudo_legal_move_bb(blockers)
        }
    }
}