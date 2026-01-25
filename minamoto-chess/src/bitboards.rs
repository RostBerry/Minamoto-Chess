// binary masks
pub const FIRST_RANK_MASK:   u64 = 0x00000000000000FF;
pub const SECOND_RANK_MASK:  u64 = 0x000000000000FF00;
pub const FOURTH_RANK_MASK:  u64 = 0x00000000FF000000;
pub const FIFTH_RANK_MASK:   u64 = 0x000000FF00000000;
pub const SEVENTH_RANK_MASK: u64 = 0x00FF000000000000;
pub const EIGHTH_RANK_MASK:  u64 = 0xFF00000000000000;

pub const NOT_FIRST_FILE_MASK: u64 = !0x8080808080808080;
pub const NOT_EIGHTH_FILE_MASK: u64 = !0x101010101010101;

/// The entire board except all edges (ranks 1, 8 and files a, h)
pub const BOARD_6X6_MASK: u64 = 0x7E7E7E7E7E7E00;

// castling masks
// contain the squares that are needed to be checked in order to make castling a legal move
pub const WHITE_KING_SIDE_CASTLING_MASK:            u64 = 0x6;
pub const BLACK_KING_SIDE_CASTLING_MASK:            u64 = 0x600000000000000;
pub const WHITE_QUEEN_SIDE_CASTLING_MASK:           u64 = 0x70;
pub const BLACK_QUEEN_SIDE_CASTLING_MASK:           u64 = 0x7000000000000000;
pub const WHITE_QUEEN_SIDE_CASTLING_KING_PATH_MASK: u64 = 0x30;
pub const BLACK_QUEEN_SIDE_CASTLING_KING_PATH_MASK: u64 = 0x3000000000000000;

pub fn get_ls1b(bitboard: u64) -> usize {
    bitboard.trailing_zeros() as usize
}

/// Returns a bitboard with a single set bit from the provided square
pub fn get_bit_from_square(square: usize) -> u64 {
    1u64 << square
}

/// Returns the provided bitboard shifted by the provided square number upwards
pub fn shift_bb(bitboard: u64, shift_amount: i8) -> u64 {
    debug_assert!(shift_amount >= -63 && shift_amount <= 63, "Shift amount out of bounds");
    if shift_amount > 0 {
        bitboard << shift_amount
    } else {
        bitboard >> -shift_amount
    }
}