use crate::{bitboards, castling, r#move::{move_record::MoveRecord, Move, MoveType}, piece::{self, *}, zobrist};
use rustc_hash::FxHashMap;

// constants
/// Since the squares on the board go from 0 to 63 included, 64 is out of bounds making it easier to catch things like king abscense
pub const INVALID_SQUARE: usize = 64;

const QUEEN_SIDE_CASTLING_ROOK_SQUARE: [usize; 2] = [7, 63];
const KING_SIDE_CASTLING_ROOK_SQUARE: [usize; 2] = [0, 56];

fn get_king_side_castling_rook_square(color: usize) -> usize {
    debug_assert!(color < 2, "Color is out of bounds");
    unsafe {
        *KING_SIDE_CASTLING_ROOK_SQUARE.get_unchecked(color)
    }
}

fn get_queen_side_castling_rook_square(color: usize) -> usize {
    debug_assert!(color < 2, "Color is out of bounds");
    unsafe {
        *QUEEN_SIDE_CASTLING_ROOK_SQUARE.get_unchecked(color)
    }
}

const KING_START_SQUARES: [usize; 2] = [3, 59];

pub fn get_king_start_square(color: usize) -> usize {
    debug_assert!(color < 2, "Color is out of bounds");
    unsafe {
        *KING_START_SQUARES.get_unchecked(color)
    }
}

const KING_SIDE_CASTLING_SQUARES: [usize; 2] = [1, 57];
const QUEEN_SIDE_CASTLING_SQUARES: [usize; 2] = [5, 61];

pub fn get_king_side_square(color: usize) -> usize {
    debug_assert!(color < 2, "Color is out of bounds");
    unsafe {
        *KING_SIDE_CASTLING_SQUARES.get_unchecked(color)
    }
}

pub fn get_queen_side_square(color: usize) -> usize {
    debug_assert!(color < 2, "Color is out of bounds");
    unsafe {
        *QUEEN_SIDE_CASTLING_SQUARES.get_unchecked(color)
    }
}

const CASTLED_KING_SIDE_ROOK_SQUARES: [usize; 2] = [2, 58];
const CASTLED_QUEEN_SIDE_ROOK_SQUARES: [usize; 2] = [4, 60];

fn get_castled_king_side_rook_square(color: usize) -> usize {
    debug_assert!(color < 2, "Color is out of bounds");
    unsafe {
        *CASTLED_KING_SIDE_ROOK_SQUARES.get_unchecked(color)
    }
}

fn get_castled_queen_side_rook_square(color: usize) -> usize {
    debug_assert!(color < 2, "Color is out of bounds");
    unsafe {
        *CASTLED_QUEEN_SIDE_ROOK_SQUARES.get_unchecked(color)
    }
}

const EMPTY_SQUARES: [(usize, usize); 64] = [(piece::INVALID_COLOR, piece::NONE); 64];
const EMPTY_PIECES: [[u64; 7]; 2] = unsafe { std::mem::zeroed() };
const DEFAULT_CASTLING_STATES: [u8; 2] = [0b11, 0b11];

/// Contains everything about the current position
pub struct Board {
    /// Contains bitboards for every piece type for each color
    /// 
    /// The bitboards at index 0 are for all pieces combined
    pieces: [[u64; 7]; 2],
    squares: [(usize, usize); 64],
    ///Can be either *WHITE* or *BLACK*
    /// 
    /// It is integer instead of bool because it is used in array indexing like in *king_square* or *piece*
    current_color: usize,
    castling_states: [u8; 2],
    is_en_passant_possible: bool,
    en_passant_pawn_square: usize,
    en_passant_capture_square: usize,
    /// Zobrist hash of the current position
    zobrist_hash: u64,
    /// History of positions with their occurrence count
    position_history: FxHashMap<u64, u8>,
    halfmoves_50_rule_counter: u8,
    move_counter: u16,
}

impl Board {
    /// Creates empty board
    pub fn empty () -> Self {
        let board = Self {
            pieces: EMPTY_PIECES,
            squares: EMPTY_SQUARES,
            current_color: WHITE,
            castling_states: DEFAULT_CASTLING_STATES,
            is_en_passant_possible: false,
            en_passant_pawn_square: INVALID_SQUARE,
            en_passant_capture_square: INVALID_SQUARE,
            zobrist_hash: 0,
            position_history: FxHashMap::default(),
            halfmoves_50_rule_counter: 0,
            move_counter: 0,
        };
        
        board
    }

    pub fn get_current_color(&self) -> usize {
        self.current_color
    }

    pub fn get_opposite_color(&self) -> usize {
        1 - self.current_color
    }

    /// Returns the square of the king of the color provided
    pub fn get_king_square(&self, color: usize) -> usize {
        debug_assert!(color < 2, "Color is out of bounds");
        debug_assert!(self.get_piece_bitboard(color, piece::KING).count_ones() == 1, "Wrong number of kings");
        unsafe {
            self.pieces.get_unchecked(color).get_unchecked(piece::KING).trailing_zeros() as usize
        }
    }

    /// Returns bitboard corresponding to the provided piece
    pub fn get_piece_bitboard(&self, color: usize, piece_type: usize) -> u64 {
        debug_assert!(piece_type < 7, "Piece type is out of bounds");
        debug_assert!(color < 2, "Color is out of bounds");
        unsafe {
            *self.pieces.get_unchecked(color).get_unchecked(piece_type)
        }
    }

    /// Returns piece standing on the provided square (or *INVALID_PIECE* if the square is empty)
    pub fn get_piece_on_square(&self, square: usize) -> (usize, usize) {
        debug_assert!(square < 64, "Square is out of bounds");
        unsafe {
            *self.squares.get_unchecked(square)
        }
    }

    /// Returns bitboard containing every single piece on the board
    pub fn get_all_occupied_squares(&self) -> u64 {
        self.get_all_occupied_squares_for_color(WHITE) | self.get_all_occupied_squares_for_color(BLACK)
    }

    /// Returns bitboard containing every single piece of the color provided
    pub fn get_all_occupied_squares_for_color(&self, color: usize) -> u64 {
        debug_assert!(color < 2, "Color is out of bounds");
        unsafe {
            *self.pieces.get_unchecked(color).get_unchecked(0)
        }
    }
    
    pub fn is_white_to_move(&self) -> bool {
        self.get_current_color() == WHITE
    }

    pub fn get_castling_state(&self, color: usize) -> u8 {
        debug_assert!(color < 2, "Color is out of bounds");
        unsafe {
            *self.castling_states.get_unchecked(color)
        }
    }

    ///Returns the castling state for the side of the color provided
    pub fn get_castling_state_mut(&mut self, color: usize) -> &mut u8 {
        debug_assert!(color < 2, "Color is out of bounds");
        unsafe {
            &mut *self.castling_states.get_unchecked_mut(color)
        }
    }

    pub fn is_en_passant_possible(&self) -> bool {
        self.is_en_passant_possible
    }

    pub fn en_passant_pawn_square(&self) -> usize {
        self.en_passant_pawn_square
    }

    pub fn en_passant_capture_square(&self) -> usize {
        self.en_passant_capture_square
    }

    pub fn get_position_history(&self) -> &FxHashMap<u64, u8> {
        &self.position_history
    }

    pub fn rule50_count(&self) -> u8 {
        self.halfmoves_50_rule_counter
    }

    pub fn get_move_counter(&self) -> u16 {
        self.move_counter
    }

    /// Updates the en passant state based on the provided square
    fn update_en_passant_state(&mut self, possible: bool, pawn_square: usize, capture_square: usize) {
        self.is_en_passant_possible = possible;
        if !possible {
            return;
        }

        self.en_passant_pawn_square = pawn_square;
        self.en_passant_capture_square = capture_square;
    }

    /// Used after every move
    pub fn switch_color(&mut self) {
        self.current_color = 1 - self.current_color;
    }

    fn delete_piece(&mut self, square: usize) {
        debug_assert!(square < 64, "Square is out of bounds");
        let inverted_bit: u64 = !bitboards::get_bit_from_square(square);

        unsafe {
            let (color, piece_type) = self.squares.get_unchecked_mut(square);
        
            *self.pieces.get_unchecked_mut(*color).get_unchecked_mut(0) &= inverted_bit;
            *self.pieces.get_unchecked_mut(*color).get_unchecked_mut(*piece_type) &= inverted_bit;

            *color = piece::INVALID_COLOR;
            *piece_type = piece::NONE;
        }
    }

    fn create_piece(&mut self, square: usize, color: usize, piece_type: usize) {
        debug_assert!(square < 64, "Square is out of bounds");
        debug_assert!(piece_type < 7, "Piece type is out of bounds");
        debug_assert!(color < 2, "Color is out of bounds");
        let bit = bitboards::get_bit_from_square(square);

        unsafe {
            let (current_color, current_piece_type) = self.squares.get_unchecked_mut(square);
            *current_color = color;
            *current_piece_type = piece_type;

            *self.pieces.get_unchecked_mut(color).get_unchecked_mut(0) |= bit;
            *self.pieces.get_unchecked_mut(color).get_unchecked_mut(piece_type) |= bit;
        }
    }

    pub fn make_move(&mut self, move_to_make: Move) -> MoveRecord {    
        let current_color = self.get_current_color();
        let opposite_color = self.get_opposite_color();
            
        let old_hash = self.zobrist_hash;
        let old_castling_states = self.castling_states;
        let old_halfmoves = self.halfmoves_50_rule_counter;
        let old_move_counter = self.move_counter;
        self.move_counter += 1 * opposite_color as u16;

        let start_square = move_to_make.start_square;
        let target_square = move_to_make.target_square;

        let (_, mut piece_type) = self.get_piece_on_square(start_square);

        debug_assert!(piece_type != piece::NONE, "There is no piece on the start square");
        // deleting the piece from its start square
        self.delete_piece(start_square);

        let captured_square = move_to_make.capture_square;
        let (_, captured_piece_type) = self.get_piece_on_square(captured_square);

        if captured_piece_type != piece::NONE {
            self.delete_piece(captured_square);
        }

        self.update_en_passant_state(false, INVALID_SQUARE, INVALID_SQUARE);

        let king_side_castling_rook_square = get_king_side_castling_rook_square(current_color);
        let queen_side_castling_rook_square = get_queen_side_castling_rook_square(current_color);

        match move_to_make.move_type {
            MoveType::PromotionQueen => piece_type = piece::QUEEN,
            MoveType::PromotionKnight => piece_type = piece::KNIGHT,
            MoveType::PromotionRook => piece_type = piece::ROOK,
            MoveType::PromotionBishop => piece_type = piece::BISHOP,
            MoveType::PawnDoubleMove => self.update_en_passant_state(true, target_square, (start_square + target_square) / 2),
            MoveType::CastlingKingSide => {
                self.delete_piece(king_side_castling_rook_square);
                self.create_piece(get_castled_king_side_rook_square(current_color), current_color, piece::ROOK);
            },
            MoveType::CastlingQueenSide => {
                self.delete_piece(queen_side_castling_rook_square);
                self.create_piece(get_castled_queen_side_rook_square(current_color), current_color, piece::ROOK);
            },
            _ => (),
        }

        self.create_piece(target_square, current_color, piece_type);

        let mut castling_state = self.get_castling_state_mut(current_color);
        if piece_type == KING {
            castling::annul(&mut castling_state);
        }

        if piece_type == ROOK {
            if start_square == king_side_castling_rook_square {
                castling::annul_king_side(&mut castling_state);
            }
            if start_square == queen_side_castling_rook_square {
                castling::annul_queen_side(&mut castling_state);
            }
        }

        let mut opposite_castling_state = self.get_castling_state_mut(opposite_color);
        if target_square == get_king_side_castling_rook_square(opposite_color) {
            castling::annul_king_side(&mut opposite_castling_state);
        } 
        if target_square == get_queen_side_castling_rook_square(opposite_color) {
            castling::annul_queen_side(&mut opposite_castling_state);
        }

        if piece_type == PAWN || captured_piece_type != piece::NONE {
            self.halfmoves_50_rule_counter = 0;
        } else {
            self.halfmoves_50_rule_counter += 1;
        }

        self.switch_color();
        
        // Update Zobrist hash after the move
        self.zobrist_hash = zobrist::calculate_hash(self);
        
        // Add new position to history
        *self.position_history.entry(self.zobrist_hash).or_insert(0) += 1;

        MoveRecord::new(
            move_to_make,
            captured_piece_type,
            self.is_en_passant_possible,
            self.en_passant_pawn_square,
            self.en_passant_capture_square,
            old_castling_states,
            old_hash,
            old_halfmoves,
            old_move_counter
        )
    }
    
    /// Undoes the move from the provided MoveInfo object
    pub fn undo_move(&mut self, move_record: MoveRecord) {
        // Remove current position from history
        if let Some(count) = self.position_history.get_mut(&self.zobrist_hash) {
            if *count > 1 {
                *count -= 1;
            } else {
                self.position_history.remove(&self.zobrist_hash);
            }
        } else {
            panic!("Position not found in history");
        }

        self.zobrist_hash = move_record.old_hash;

        self.halfmoves_50_rule_counter = move_record.old_halfmoves;
        self.move_counter = move_record.old_move_counter;
        
        let mov = move_record.mov;
        let start_square = mov.start_square;
        let target_square = mov.target_square;
        let captured_piece_type = move_record.captured_piece_type;
        let captured_square = mov.capture_square;
        self.castling_states = move_record.old_castling_states;
        self.is_en_passant_possible = move_record.is_en_passant_possible;
        self.en_passant_pawn_square = move_record.en_passant_pawn_square;
        self.en_passant_capture_square = move_record.en_passant_capture_square;

        let (_, mut moved_piece_type) = self.get_piece_on_square(target_square);

        debug_assert!(moved_piece_type != piece::NONE, "There is no piece on the target square");

        self.switch_color();

        let current_color = self.get_current_color();

        // deleting the piece from the target square
        self.delete_piece(target_square);


        match mov.move_type {
            MoveType::PromotionQueen | MoveType::PromotionKnight | MoveType::PromotionRook | MoveType::PromotionBishop => {
                moved_piece_type = piece::PAWN;
            },
            MoveType::CastlingKingSide => {
                self.delete_piece(get_castled_king_side_rook_square(current_color));
                self.create_piece(get_king_side_castling_rook_square(current_color), current_color, piece::ROOK);
            },
            MoveType::CastlingQueenSide => {
                self.delete_piece(get_castled_queen_side_rook_square(current_color));
                self.create_piece(get_queen_side_castling_rook_square(current_color), current_color, piece::ROOK);
            },
            _ => (),
        }

        self.create_piece(start_square, current_color, moved_piece_type);

        if captured_piece_type == piece::NONE {
            return;
        }

        self.create_piece(captured_square, self.get_opposite_color(), captured_piece_type, );
    
    }
    
    /// Checks if the current position has occurred three or more times
    #[inline]
    pub fn is_draw_by_repetition(&self) -> bool {
        if let Some(count) = self.position_history.get(&self.zobrist_hash) {
            *count >= 3
        } else {
            false
        }
    }

    #[inline]
    pub fn is_draw_by_50_moves_rule(&self) -> bool {
        self.halfmoves_50_rule_counter >= 50
    }

    #[inline]
    pub fn is_draw_by_material(&self) -> bool {
        !(self.enough_material(WHITE) || self.enough_material(BLACK))
    }

    fn enough_material(&self, color: usize) -> bool {
        let mut enough = false;

        enough |= self.get_piece_bitboard(color, piece::PAWN) != 0;
        enough |= self.get_piece_bitboard(color, piece::ROOK) != 0;
        enough |= self.get_piece_bitboard(color, piece::QUEEN) != 0;
        enough |= self.get_piece_bitboard(color, piece::BISHOP).count_ones() > 1;

        enough
    }
    
    /// Returns the Zobrist hash of the current position
    #[inline]
    pub fn get_zobrist_hash(&self) -> u64 {
        self.zobrist_hash
    }
}

#[cfg(feature = "__internal_api")]
impl Board {
    pub fn create_piece_public(&mut self, square: usize, color: usize, piece_type: usize) {
        self.create_piece(square, color, piece_type);
    }

    pub fn rule50_count_mut(&mut self) -> &mut u8 {
        &mut self.halfmoves_50_rule_counter
    }

    pub fn get_move_counter_mut(&mut self) -> &mut u16 {
        &mut self.move_counter
    }

    pub fn update_en_passant_state_public(&mut self, possible: bool, pawn_square: usize, capture_square: usize) {
        self.update_en_passant_state(possible, pawn_square, capture_square);
    }

    pub fn get_zobrist_hash_mut(&mut self) -> &mut u64 {
        &mut self.zobrist_hash
    }

    pub fn get_position_history_mut(&mut self) -> &mut FxHashMap<u64, u8> {
        &mut self.position_history
    }
}