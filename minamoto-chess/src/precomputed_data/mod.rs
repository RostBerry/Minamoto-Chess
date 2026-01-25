pub mod magic_bitboard_data;
pub mod bitboard_data_gen;
pub mod square_data;
pub mod magic_score;
mod bitboard_data;
pub mod magic_lookup_table;
pub mod square_magic;
pub mod magic_bitboard_gen;
pub mod magic_lookup_moves;
pub mod magic_loader;

use once_cell::sync::Lazy;

use magic_lookup_table::MagicLookupTable;
use square_data::SquareData;

use crate::piece;

pub static BITBOARD_DATA: Lazy<Box<bitboard_data::BitboardData>> = Lazy::new(bitboard_data::BitboardData::new);
pub static SQUARE_DATA: Lazy<Box<SquareData>> = Lazy::new(SquareData::new);
pub static ROOK_MAGIC_LOOKUP_TABLE: Lazy<Box<MagicLookupTable>> = Lazy::new(|| MagicLookupTable::new(SLIDER_ROOK_INDEX));
pub static BISHOP_MAGIC_LOOKUP_TABLE: Lazy<Box<MagicLookupTable>> = Lazy::new(|| MagicLookupTable::new(SLIDER_BISHOP_INDEX));

pub const SLIDER_TYPES: [usize; 2] = [piece::BISHOP, piece::ROOK];

pub fn get_slider_type(slider_type_index: usize) -> usize {
    debug_assert!(slider_type_index < 2, "Slider type index out of bounds");
    unsafe {
        *SLIDER_TYPES.get_unchecked(slider_type_index)
    }
}

pub const SLIDER_BISHOP_INDEX: usize = 0;
pub const SLIDER_ROOK_INDEX: usize = 1;