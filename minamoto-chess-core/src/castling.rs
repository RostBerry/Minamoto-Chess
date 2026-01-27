pub const KING_SIDE: u8 = 0b01;
pub const QUEEN_SIDE: u8 = 0b10;
pub const BOTH_SIDES: u8 = 0b11;
pub const NO_SIDES: u8 = 0b00;

pub fn can_king_side(castling_state: u8) -> bool {
    castling_state & KING_SIDE == KING_SIDE
}

pub fn can_queen_side(castling_state: u8) -> bool {
    castling_state & QUEEN_SIDE == QUEEN_SIDE
}

pub fn can_any(castling_state: u8) -> bool {
    castling_state != NO_SIDES
}

pub fn annul_king_side(castling_state: &mut u8) {
    *castling_state &= QUEEN_SIDE;
}

pub fn annul_queen_side(castling_state: &mut u8) {
    *castling_state &= KING_SIDE;
}

pub fn annul(castling_state: &mut u8) {
    *castling_state = NO_SIDES;
}