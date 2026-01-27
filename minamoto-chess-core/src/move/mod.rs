pub mod move_record;

#[derive(Clone, Copy)]
pub enum MoveType {
    Regular,
    PawnDoubleMove,
    PromotionQueen,
    PromotionKnight,
    PromotionRook,
    PromotionBishop,
    CastlingKingSide,
    CastlingQueenSide
}

#[derive(Clone, Copy)]
pub struct Move {
    pub start_square: usize,
    pub target_square: usize,
    pub capture_square: usize,
    pub move_type: MoveType,
}

impl Move {
    pub fn new(
        start_square: usize, 
        target_square: usize, 
        capture_square: usize, 
        move_type: MoveType
    ) -> Self {
        Self {
            start_square,
            target_square,
            capture_square,
            move_type
        }
    }

    pub fn is_promotion(&self) -> bool {
        match self.move_type {
            MoveType::PromotionQueen | MoveType::PromotionKnight | MoveType::PromotionRook | MoveType::PromotionBishop => true,
            _ => false
        }
    }

    pub fn is_castling(&self) -> bool {
        match self.move_type {
            MoveType::CastlingKingSide | MoveType::CastlingQueenSide => true,
            _ => false   
        }
    }
}