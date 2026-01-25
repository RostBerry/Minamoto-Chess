pub fn can_king_side(castling_state: u8) -> bool {
    castling_state & 0b01 == 0b01
}

pub fn can_queen_side(castling_state: u8) -> bool {
    castling_state & 0b10 == 0b10
}

pub fn can_any(castling_state: u8) -> bool {
    castling_state != 0
}

pub fn annul_king_side(castling_state: &mut u8) {
    *castling_state &= 0b10;
}

pub fn annul_queen_side(castling_state: &mut u8) {
    *castling_state &= 0b01;
}

pub fn annul(castling_state: &mut u8) {
    *castling_state = 0;
}