/*Contains data that is either precalculated on the init or constant
to allow to use system's resources on something more important*/

use std::cmp::min;

/// Returns the rank (row 0-7) of the provided square
pub fn get_rank_from_square(square: usize) -> usize {
    square >> 3
}

/// Returns the file (column 0-7) of the provided square
pub fn get_file_from_square(square: usize) -> usize {
    square & 0b111
}

/// Used to give clearer instructions to the data loaders
/// 
/// Since there are no actual squares and the entire board with all pieces
/// is just a u64 number, making a move happens by changing the pieces'
/// bit position into a different one. And because pieces often can't 
/// teleport to a random position but only go orthogonally/diagonally, these 
/// directions are contained here
/// 
/// !!! This enum is used only on init stage for data precalculation 
/// purposes. Using it in bitboard offset generation would not work 
const MOVE_OFFSETS: [isize; 8] = [8,      -8,      -1,       1,       7,       9,      -9,      -7];
//                             up     down     left     right    upleft  upright  downleft downright
///Returns the amount of bits to shift to get to 1 square in a certain direction
pub fn get_move_offset(direction_index: usize) -> isize {
    debug_assert!(direction_index < 8, "Direction index out of bounds");
    unsafe {
        *MOVE_OFFSETS.get_unchecked(direction_index)
    }
}

/// Same as *MOVE_OFFSETS* but for knight
/// 
/// Used only to precalculate *SQUARES_FOR_KNIGHT_BB*
const KNIGHT_MOVE_OFFSETS: [i8; 8] = [15, 17, 6, 10, -10, -6, -17, -15];


/// Contains necessary square data calculated on init
/// 
/// The move bitboards of ray pieces is stored into *MagicLookupTable*
pub struct SquareData {
    /// Squares from every square to every board edge. Goes like *squares_to_edge\[square]\[direction_index]*
    /// 
    /// The *direction_index* is a corresponding index of *MOVE_OFFSETS*
    squares_to_edge: [[usize; 8]; 64],
    /// Bitboards containing king moves for an empty board for every square
    king_square_bbs: [u64; 64],
    /// Bitboards containing knight moves for an empty board for every square
    knight_square_bbs: [u64; 64],
    /// Bitboards containing lines from every square to every square (or zeros if no line is possible)
    square_to_square_line_bbs: [[u64; 64]; 64],
    /// Contains bitboards with all files, ranks and diagonals for each square on the board
    /// 
    /// 0: Ranks, 1: Files, 2: Diagonals, 3: Anti-diagonals
    files_ranks_diagonals: [[u64; 64]; 4],
}

impl SquareData {
    #[cold]
    pub fn new() -> Box<Self> {
        let squares_to_edge = Self::load_squares_to_edge();

        let mut square_data = SquareData {
            squares_to_edge,
            king_square_bbs: [0; 64],
            knight_square_bbs: [0; 64],
            square_to_square_line_bbs: [[0; 64]; 64],
            files_ranks_diagonals: [[0; 64]; 4],
        };

        square_data.load_king_squares_bb();
        square_data.load_knight_squares_bb();
        square_data.load_lines_bb();
        square_data.load_files_ranks_diagonals();

        Box::new(square_data)
    }

    /// Returns the amount of squares from any square to any edge of the board
    pub fn get_squares_to_edge(&self, square: usize, direction_index: usize) -> usize {
        debug_assert!(square < 64, "Square index out of bounds");
        debug_assert!(direction_index < 8, "Direction index out of bounds");
        unsafe {
            *self.squares_to_edge.get_unchecked(square).get_unchecked(direction_index)
        }
    }
    /// Returns bitboard with all squares a king can go to on an empty board from the provided start square
    pub fn get_bb_for_king(&self, square: usize) -> u64 {
        debug_assert!(square < 64, "Square index out of bounds");
        unsafe {
            *self.king_square_bbs.get_unchecked(square)
        }
    }
    /// Returns bitboard with all squares a knight can go to pseudolegally from provided start square
    pub fn get_bb_for_knight(&self, square: usize) -> u64{
        debug_assert!(square < 64, "Square index out of bounds");
        unsafe {
            *self.knight_square_bbs.get_unchecked(square)
        }
    }
    /// Returns a bitboard containing all squares in line from start to target squares including
    /// 
    ///  Returns zero if no line was found
    pub fn get_bb_line(&self, start_square: usize, target_square: usize) -> u64 {
        debug_assert!(start_square < 64, "Start square index out of bounds");
        debug_assert!(target_square < 64, "Target square index out of bounds");
        unsafe {
            *self.square_to_square_line_bbs.get_unchecked(start_square).get_unchecked(target_square)
        }
    }

    pub fn get_file_rank_diagonal_mask(&self, square: usize, direction_index: usize) -> u64 {
        debug_assert!(square < 64, "Square index out of bounds");
        debug_assert!(direction_index < 4, "Direction index out of bounds");
        unsafe {
            *self.files_ranks_diagonals.get_unchecked(direction_index).get_unchecked(square)
        }
    }

    #[cold]
    fn load_squares_to_edge() -> [[usize; 8]; 64] {
        let mut squares_to_edge: [[usize; 8]; 64] = [[0; 8]; 64];
        let mut to_north;
        let mut to_south;
        let mut to_west;
        let mut to_east;
        let mut square;

        for y in 0..8 {
            for x in 0..8 {
                to_north = 7 - y;
                to_south = y;
                to_west = x;
                to_east = 7 - x;

                square = x + y * 8;

                squares_to_edge[square][0] = to_north;
                squares_to_edge[square][1] = to_south;
                squares_to_edge[square][2] = to_west;
                squares_to_edge[square][3] = to_east;
                squares_to_edge[square][4] = min(to_north, to_west);
                squares_to_edge[square][5] = min(to_north, to_east);
                squares_to_edge[square][6] = min(to_south, to_west);
                squares_to_edge[square][7] = min(to_south, to_east);
            }
        }

        squares_to_edge
    }

    #[cold]
    fn load_king_squares_bb(&mut self) {
        for y in 0..8 {
            for x in 0..8 {
                let square = x + y * 8;
                let mut available_squares_bb: u64 = 0;

                for direction_index in 0..8 {
                    if self.get_squares_to_edge(square, direction_index) != 0 {
                        available_squares_bb |= 1 << ((square as isize + get_move_offset(direction_index)) as usize);
                    }
                }

                self.king_square_bbs[square] = available_squares_bb;
            }
        }
    }

    #[cold]
    fn load_knight_squares_bb(&mut self) {

        for y in 0..8 {
            for x in 0..8 {
                let square: i8 = x + y * 8;
                let square_rank: i8 = square & 0b111;
                let square_file: i8 = square >> 3;
                let mut available_moves: u64 = 0;

                for offset in KNIGHT_MOVE_OFFSETS {
                    let target_square: i8 = square + offset;
                    let target_square_rank: i8 = target_square & 0b111;
                    let target_square_file: i8 = target_square >> 3;

                    if target_square_file >= 0 && target_square_file < 8 && target_square_rank >= 0 && target_square_rank < 8
                    && (square_rank as i8 - target_square_rank as i8).abs() < 3 && (square_file as i8 - target_square_file as i8).abs() < 3 {
                        available_moves |= 1 << (target_square as usize);
                    }
                }
                self.knight_square_bbs[square as usize] = available_moves;
            }
        }
    }

    #[cold]
    fn load_lines_bb(&mut self) {
        for start_square in 0..64 {
            for target_square in 0..64 {
                if start_square == target_square {
                    continue;
                }

                let mut is_line_found: bool = false;

                for direction_index in 0..8 {
                    let mut line: u64 = 0;

                    for multiplier in 0..self.get_squares_to_edge(start_square, direction_index) {
                        let allowed_target_square = (start_square as isize + (get_move_offset(direction_index) * (multiplier + 1) as isize)) as usize;
                        line |= 1 << (allowed_target_square);
                        if target_square == allowed_target_square {
                            self.square_to_square_line_bbs[start_square][target_square] = 1 << (start_square) as u64 | line;
                            is_line_found = true;
                            break;
                        }
                    }
                    if is_line_found {
                        break;
                    }
                }
                if !is_line_found {
                    self.square_to_square_line_bbs[start_square][target_square] = 0;
                }
            }
        }
    }

    #[cold]
    /// Calculates bitboards with files, ranks, diagonals and anti-diagonals for each square on the board
    pub fn load_files_ranks_diagonals(&mut self)  {
        for square in 0..64 {
            let rank = get_rank_from_square(square);
            let file = get_file_from_square(square);
            for i in 0..8 {
                self.files_ranks_diagonals[0][square] |= 1 << (file + i * 8);
                self.files_ranks_diagonals[1][square] |= 1 << (rank * 8 + i);
            }

            let (mut r, mut f) = (rank as i8, file as i8);

            while r >= 0 && f >= 0 {
                self.files_ranks_diagonals[2][square] |= 1 << ((r * 8 + f) as usize);
                r -= 1;
                f -= 1;
            }

            let (mut r, mut f) = (rank as i8, file as i8);
            while r < 8 && f < 8 {
                self.files_ranks_diagonals[2][square] |= 1 << ((r * 8 + f) as usize);
                r += 1;
                f += 1;
            }

            let (mut r, mut f) = (rank as i8, file as i8);
            while r >= 0 && f < 8 {
                self.files_ranks_diagonals[3][square] |= 1 << ((r * 8 + f) as usize);
                r -= 1;
                f += 1;
            }

            let (mut r, mut f) = (rank as i8, file as i8);
            while r < 8 && f >= 0 {
                self.files_ranks_diagonals[3][square] |= 1 << ((r * 8 + f) as usize);
                r += 1;
                f -= 1;
            }
        }
    }
}