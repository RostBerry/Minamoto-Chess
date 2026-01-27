use minamoto_chess_core::{bitboards, board::Board, piece};
/* Contains all values either constant or calculated on init, which are used only for better expierence 
on the user side like more human representation of the squares (using *e1* instead of *3*) etc. */
use phf::phf_map;

// Decorative strings
// Used when the board is printed
const DECORATIVE_ROW:               &str = "+-----+-----+-----+-----+-----+-----+-----+-----+";
const DECORATIVE_LETTERS_ROW:       &str = "   a     b     c     d     e     f     g     h   ";
const EMPTY_SQUARE_ON_BITBOARD:     &str = ".";
const OCCUPIED_SQUARE_ON_BITBOARD:  &str = "1";

// Game positions
/// Position the regular chess game starts from
pub const DEFAULT_FEN: &str             = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
/// Position used for pseudo-legal move generation testing
pub const PERFT_FEN: &str               = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
/// Position used for legal move generation and move search testing
pub const MATE_IN_2_FEN: &str           = "kbK5/pp6/1P6/8/8/8/8/R7 w - -";

pub const ROW_TO_BOARD_LETTER: [char; 8] = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'];
/// Outputs a letter on the board from the provided row number, for user's comfort
pub fn get_board_row_letter(row: usize) -> char {
    ROW_TO_BOARD_LETTER[row]
}

static FEN_SYM_TO_PIECE: phf::Map<char, (usize, usize)> = phf_map! {
    ' ' => (piece::INVALID_COLOR, piece::NONE),
    'K' => (piece::WHITE, piece::KING),
    'k' => (piece::BLACK, piece::KING),
    'P' => (piece::WHITE, piece::PAWN),
    'p' => (piece::BLACK, piece::PAWN),
    'N' => (piece::WHITE, piece::KNIGHT),
    'n' => (piece::BLACK, piece::KNIGHT),
    'B' => (piece::WHITE, piece::BISHOP),
    'b' => (piece::BLACK, piece::BISHOP),
    'R' => (piece::WHITE, piece::ROOK),
    'r' => (piece::BLACK, piece::ROOK),
    'Q' => (piece::WHITE, piece::QUEEN),
    'q' => (piece::BLACK, piece::QUEEN),
};
/// outputs a corresponding piece to the FEN character provided
pub fn get_piece_from_fen(fen_sym: &char) -> &(usize, usize) {
    match FEN_SYM_TO_PIECE.get(fen_sym) {
        Some(received_piece) => {
            return received_piece;
        },
        None => {
            panic!("Something went wrong with pieces' names");
        }
    }
}

const SQUARE_TO_NAME: [&'static str; 64] = [
    "h1", "g1", "f1", "e1", "d1", "c1", "b1", "a1",
    "h2", "g2", "f2", "e2", "d2", "c2", "b2", "a2",
    "h3", "g3", "f3", "e3", "d3", "c3", "b3", "a3",
    "h4", "g4", "f4", "e4", "d4", "c4", "b4", "a4",
    "h5", "g5", "f5", "e5", "d5", "c5", "b5", "a5",
    "h6", "g6", "f6", "e6", "d6", "c6", "b6", "a6",
    "h7", "g7", "f7", "e7", "d7", "c7", "b7", "a7",
    "h8", "g8", "f8", "e8", "d8", "c8", "b8", "a8",
];
/// Outputs a corresponding name like *e1*, *g6* to the provided square
pub fn get_square_name(square: usize) -> &'static str {
    SQUARE_TO_NAME[square]
}

static NAME_TO_SQUARE: phf::Map<&'static str, usize> = phf_map! {
    "h1" => 0,  "g1" => 1,  "f1" => 2,  "e1" => 3,  "d1" => 4,  "c1" => 5,  "b1" => 6,  "a1" => 7,
    "h2" => 8,  "g2" => 9,  "f2" => 10, "e2" => 11, "d2" => 12, "c2" => 13, "b2" => 14, "a2" => 15,
    "h3" => 16, "g3" => 17, "f3" => 18, "e3" => 19, "d3" => 20, "c3" => 21, "b3" => 22, "a3" => 23,
    "h4" => 24, "g4" => 25, "f4" => 26, "e4" => 27, "d4" => 28, "c4" => 29, "b4" => 30, "a4" => 31,
    "h5" => 32, "g5" => 33, "f5" => 34, "e5" => 35, "d5" => 36, "c5" => 37, "b5" => 38, "a5" => 39,
    "h6" => 40, "g6" => 41, "f6" => 42, "e6" => 43, "d6" => 44, "c6" => 45, "b6" => 46, "a6" => 47,
    "h7" => 48, "g7" => 49, "f7" => 50, "e7" => 51, "d7" => 52, "c7" => 53, "b7" => 54, "a7" => 55,
    "h8" => 56, "g8" => 57, "f8" => 58, "e8" => 59, "d8" => 60, "c8" => 61, "b8" => 62, "a8" => 63,
};
/// Outputs a corresponding square to the provided square name
pub fn get_square_from_name(square_name: &str) -> usize {
    match NAME_TO_SQUARE.get(square_name) {
        Some(square) => {
            return *square;
        },
        None => {
            panic!("wrong indexation");
        }
    }
}

const PIECE_NAMES: [[char; 6]; 2] = [
    [ 'K', 'P', 'N', 'B', 'R', 'Q' ],
    [ 'k', 'p', 'n', 'b', 'r', 'q' ]
];
pub fn piece_to_fen_sym(color: usize, piece_type: usize) -> char {
    if piece_type == piece::NONE {
        return ' ';
    }
    PIECE_NAMES[color][piece_type - 1]
}

/// Prints the provided bitboard into more human representation (*.* for empty squares, *1* for occupied squares)
pub fn print_bitboard(bitboard: u64) {
    println!("   {}", DECORATIVE_LETTERS_ROW);
    println!("   {}", DECORATIVE_ROW);
    for y in (0..=7).rev() {
        print!(" {} ", y + 1);
        print!("|");
        for x in (0..8).rev() {
            let square = x + y * 8;
            print!(
                "  {}  |", 
                if bitboard & bitboards::get_bit_from_square(square) != 0 {OCCUPIED_SQUARE_ON_BITBOARD} 
                else {EMPTY_SQUARE_ON_BITBOARD}
            );
        }
        println!(" {} ", y + 1);
        println!("   {}", DECORATIVE_ROW);
    }
    println!("   {}", DECORATIVE_LETTERS_ROW);
}

/// Prints the provided board into a readable FEN notated chess position
pub fn print_board(board: &Board) {
    println!("   {}", DECORATIVE_LETTERS_ROW);
    println!("   {}", DECORATIVE_ROW);
    for y in (0..=7).rev() {
        print!(" {} ", y + 1);
        print!("|");
        for x in (0..8).rev() {
            let square = x + y * 8;
            let piece = board.get_piece_on_square(square);
            print!("  {}  |", piece_to_fen_sym(piece.0, piece.1));
        }
        println!(" {} ", y + 1);
        println!("   {}", DECORATIVE_ROW);
    }
    println!("   {}", DECORATIVE_LETTERS_ROW);
}