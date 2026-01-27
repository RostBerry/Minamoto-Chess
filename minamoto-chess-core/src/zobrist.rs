use once_cell::sync::Lazy;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use crate::piece;
use crate::board::Board;

pub struct ZobristKeys {
    // [color][piece_type][square]
    piece_keys: [[[u64; 64]; 7]; 2],
    // Key for side to move (black)
    side_to_move_key: u64,
    // [color][rights] - rights can be 0-3 (representing the 2 bits of castling rights)
    castling_keys: [[u64; 4]; 2],
    // [file] - for en passant
    en_passant_keys: [u64; 8],
}

impl ZobristKeys {
    fn new() -> Self {
        let mut rng = StdRng::seed_from_u64(0xCAFEBABE); // Constant seed for deterministic behavior
        
        let mut piece_keys = [[[0; 64]; 7]; 2];
        let mut castling_keys = [[0; 4]; 2];
        let mut en_passant_keys = [0; 8];
        
        // Generate random numbers for pieces on squares
        for color in 0..2 {
            for piece_type in 0..7 {
                for square in 0..64 {
                    piece_keys[color][piece_type][square] = rng.random();
                }
            }
        }
        
        // Generate random numbers for castling rights
        for color in 0..2 {
            for rights in 0..4 {
                castling_keys[color][rights] = rng.random();
            }
        }
        
        // Generate random numbers for en passant files
        for file in 0..8 {
            en_passant_keys[file] = rng.random();
        }
        
        // Generate random number for side to move
        let side_to_move_key = rng.random();
        
        Self {
            piece_keys,
            side_to_move_key,
            castling_keys,
            en_passant_keys,
        }
    }
}

pub static ZOBRIST_KEYS: Lazy<ZobristKeys> = Lazy::new(|| ZobristKeys::new());

/// Calculate the Zobrist hash for the current board position
pub fn calculate_hash(board: &Board) -> u64 {
    let mut hash: u64 = 0;
    
    // Hash pieces
    for square in 0..64 {
        let (color, piece_type) = board.get_piece_on_square(square);
        if piece_type != piece::NONE {
            hash ^= ZOBRIST_KEYS.piece_keys[color][piece_type][square];
        }
    }
    
    // Hash side to move
    if board.get_current_color() == 1 {  // If black to move
        hash ^= ZOBRIST_KEYS.side_to_move_key;
    }
    
    // Hash castling rights
    for color in 0..2 {
        let rights = board.get_castling_state(color) as usize;
        hash ^= ZOBRIST_KEYS.castling_keys[color][rights];
    }
    
    // Hash en passant
    if board.is_en_passant_possible() {
        let file = board.en_passant_pawn_square() % 8;
        hash ^= ZOBRIST_KEYS.en_passant_keys[file];
    }
    
    hash
}

/// Update the given hash incrementally after a piece move
pub fn update_hash_piece_move(hash: u64, piece_color: usize, piece_type: usize, 
                           from_square: usize, to_square: usize) -> u64 {
    let mut new_hash = hash;
    
    // XOR out the piece from source square
    new_hash ^= ZOBRIST_KEYS.piece_keys[piece_color][piece_type][from_square];
    // XOR in the piece at destination square
    new_hash ^= ZOBRIST_KEYS.piece_keys[piece_color][piece_type][to_square];
    
    new_hash
}
