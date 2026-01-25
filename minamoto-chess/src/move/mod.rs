pub mod move_record;

use std::fmt::Display;

use crate::{bitboards, board::{self, Board}, board_representation::{get_piece_from_fen, get_square_from_name, get_square_name, piece_to_fen_sym}, piece};

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

// #[derive(Clone)]
pub struct UciMove {
    move_type: MoveType,
    start_square: usize,
    target_square: usize
}

pub enum UciMoveCreationResult {
    Success(UciMove),
    Failure
}

impl UciMove {
    pub fn new(move_type: MoveType, start_square: usize, target_square: usize) -> Self {
        Self {
            move_type,
            start_square,
            target_square
        }
    }

    /// Accepts a UCI move (e.g e2e4, e7e8q) as a string and returns a UciMove if the move is valid
    pub fn from_str(uci: &str) -> UciMoveCreationResult {
        if uci.len() >= 4 && uci.len() <= 5 {
            let start_square = get_square_from_name(&uci[0..2]);
            let target_square = get_square_from_name(&uci[2..4]);
            let mut move_type = MoveType::Regular;
            if uci.len() == 5 {
                let promotion = get_piece_from_fen(
                    &uci.chars().nth(4).expect("Something with move from uci")
                ).1;

                move_type = match promotion {
                    piece::QUEEN => MoveType::PromotionQueen,
                    piece::KNIGHT => MoveType::PromotionKnight,
                    piece::ROOK => MoveType::PromotionRook,
                    piece::BISHOP => MoveType::PromotionBishop,
                    _ => MoveType::Regular
                };
            }
            return UciMoveCreationResult::Success(UciMove::new(move_type, start_square, target_square));
        }
        UciMoveCreationResult::Failure
    }

    pub fn is_promotion(&self) -> bool {
        match &self.move_type {
            MoveType::PromotionQueen | MoveType::PromotionKnight | MoveType::PromotionRook | MoveType::PromotionBishop => true,
            _ => false
        }
    }
}

impl Display for UciMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "{}{}{}", 
            get_square_name(self.start_square), 
            get_square_name(self.target_square), 
            match self.move_type {
                MoveType::PromotionQueen => piece_to_fen_sym(piece::BLACK, piece::QUEEN).to_string(),
                MoveType::PromotionKnight => piece_to_fen_sym(piece::BLACK, piece::KNIGHT).to_string(),
                MoveType::PromotionRook => piece_to_fen_sym(piece::BLACK, piece::ROOK).to_string(),
                MoveType::PromotionBishop => piece_to_fen_sym(piece::BLACK, piece::BISHOP).to_string(),
                _ => String::new()
            }
        )
    }
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

    pub fn from_uci(mov: UciMove, board: &Board) -> Self {
        let start_square = mov.start_square;
        let target_square = mov.target_square;
        let target_square_bb = bitboards::get_bit_from_square(target_square);
        let (color, piece_type) = board.get_piece_on_square(start_square);
        let is_capture = target_square_bb & board.get_all_occupied_squares() != 0;
        let mut capture_square = target_square;

        if is_capture && board.is_en_passant_possible() && target_square_bb == bitboards::get_bit_from_square(board.en_passant_capture_square()) {
            capture_square = board.en_passant_pawn_square();
        }

        let mut move_type = mov.move_type;

        if piece_type == piece::KING && start_square == board::get_king_start_square(color) {
            if target_square == board::get_king_side_square(color) {
                move_type = MoveType::CastlingKingSide;
            } else if target_square == board::get_queen_side_square(color) {
                move_type = MoveType::CastlingQueenSide;
            }
        }

        if piece_type == piece::PAWN && ((target_square as i32) - (start_square as i32)).abs() == 16 {
            move_type = MoveType::PawnDoubleMove;
        }

        Self {
            start_square,
            target_square,
            capture_square,
            move_type
        }
    }
}

impl Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f, "{}{}{}", 
            get_square_name(self.start_square), 
            get_square_name(self.target_square), 
            match self.move_type {
                MoveType::PromotionQueen => piece_to_fen_sym(piece::BLACK, piece::QUEEN).to_string(),
                MoveType::PromotionKnight => piece_to_fen_sym(piece::BLACK, piece::KNIGHT).to_string(),
                MoveType::PromotionRook => piece_to_fen_sym(piece::BLACK, piece::ROOK).to_string(),
                MoveType::PromotionBishop => piece_to_fen_sym(piece::BLACK, piece::BISHOP).to_string(),
                _ => String::new()
            }
        )
    }
}