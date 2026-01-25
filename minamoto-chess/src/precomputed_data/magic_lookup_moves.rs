use std::mem::MaybeUninit;

use super::{magic_bitboard_data, square_magic::SquareMagic};

pub struct MagicLookupMoves {
    relevant_occupancy: u64,
    pseudo_legal_move_patterns: Box<[u64]>,
    square_magic: SquareMagic,
}

impl MagicLookupMoves {
    #[cold]
    pub fn new(pseudo_legal_move_patterns: Vec<u64>, relevant_occupancy: u64, magic: SquareMagic) -> Self {
        let length = pseudo_legal_move_patterns.len();
        let mut array: Box<[MaybeUninit<u64>]> = {
            let mut v = Vec::with_capacity(length);
            unsafe {
                v.set_len(length);
                v.into_boxed_slice().try_into().unwrap_unchecked()
            }
        };

        for (i, pattern) in pseudo_legal_move_patterns.into_iter().enumerate() {
            array[i].write(pattern);
        }

        let initialized_array = unsafe {
            std::mem::transmute::<Box<[MaybeUninit<u64>]>, Box<[u64]>>(array)
        };

        Self {
            relevant_occupancy,
            pseudo_legal_move_patterns: initialized_array,
            square_magic: magic,
        }
    }

    pub fn get_pseudo_legal_move_patterns(&self) -> &[u64] {
        &self.pseudo_legal_move_patterns
    }

    pub fn get_relevant_occupancy(&self) -> u64 {
        self.relevant_occupancy
    }

    pub fn get_square_magic(&self) -> &SquareMagic {
        &self.square_magic
    }

    pub fn get_pseudo_legal_move_bb(&self, blockers: u64) -> u64 {
        let relevant_occupancy = self.get_relevant_occupancy();
        let square_magic = self.get_square_magic();

        let blocker_pattern = blockers & relevant_occupancy;
        let lookup_index = magic_bitboard_data::generate_lookup_table_index(
            square_magic.get_magic(), 
            square_magic.get_shift(), 
            blocker_pattern, 
            relevant_occupancy,
        );

        unsafe {
            *self.get_pseudo_legal_move_patterns().get_unchecked(lookup_index)
        }
    }
}