//type constants
pub const NONE:   usize = 0;
pub const KING:   usize = 0b001;
pub const PAWN:   usize = 0b010;
pub const KNIGHT: usize = 0b011;
pub const BISHOP: usize = 0b100;
pub const ROOK:   usize = 0b101;
pub const QUEEN:  usize = 0b110;

//color constants
pub const WHITE: usize = 0;
pub const BLACK: usize = 1;
/// Used combined with *NONE* piece type
pub const INVALID_COLOR: usize = 2;