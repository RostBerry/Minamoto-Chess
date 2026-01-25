use crate::{bitboards, board::{Board, INVALID_SQUARE}, board_representation, piece, precomputed_data};

// constants
// prevents bits from wrapping around the board when shifting. *(move_to_left, move_to_right)*
static RELEVANT_PAWN_ATTACKS_MASK_1: [u64; 2] = [bitboards::NOT_FIRST_FILE_MASK, bitboards::NOT_EIGHTH_FILE_MASK];
static RELEVANT_PAWN_ATTACKS_MASK_2: [u64; 2] = [bitboards::NOT_EIGHTH_FILE_MASK, bitboards::NOT_FIRST_FILE_MASK];

fn get_relevant_pawn_attacks_mask_1(color: usize) -> u64 {
    debug_assert!(color < 2, "Color out of bounds");
    unsafe {
        *RELEVANT_PAWN_ATTACKS_MASK_1.get_unchecked(color)
    }
}

fn get_relevant_pawn_attacks_mask_2(color: usize) -> u64 {
    debug_assert!(color < 2, "Color out of bounds");
    unsafe {
        *RELEVANT_PAWN_ATTACKS_MASK_2.get_unchecked(color)
    }
}

// determines what side is left/right
static PAWN_ATTACK_SHIFT_OFFSETS_1: [i8; 2] = [9, -9];
static PAWN_ATTACK_SHIFT_OFFSETS_2: [i8; 2] = [7, -7];

fn get_pawn_attack_shift_offset_1(color: usize) -> i8 {
    debug_assert!(color < 2, "Color out of bounds");
    unsafe {
        *PAWN_ATTACK_SHIFT_OFFSETS_1.get_unchecked(color)
    }
}

fn get_pawn_attack_shift_offset_2(color: usize) -> i8 {
    debug_assert!(color < 2, "Color out of bounds");
    unsafe {
        *PAWN_ATTACK_SHIFT_OFFSETS_2.get_unchecked(color)
    }
}

static EN_PASSANT_RANK_MASK: [u64; 2] = [bitboards::FOURTH_RANK_MASK, bitboards::FIFTH_RANK_MASK];

fn get_en_passant_rank_mask(color: usize) -> u64 {
    debug_assert!(color < 2, "Color out of bounds");
    unsafe {
        *EN_PASSANT_RANK_MASK.get_unchecked(color)
    }
}

/// Calculates every attack from every enemy piece 
/// such as direct attacks and pins.
/// 
/// Gets called before the move generation, all attacks are created from 
/// enemy's perspective
pub struct AttackCalculator {
    /// Bitboard with every square when the king obviously can't go 
    /// because he will be immediately hit by some enemy piece
    pub squares_in_attack_bb: [u64; 7],
    /// Bitboard with every square the player can go to to avoid being in check 
    /// without moving the king
    /// 
    /// If in double check or not in check, this bitboard will be equal to 0
    pub check_block_bb: u64,
    /// Bitboards containing every piece that is restricted to a certain axis 
    /// not to reveal a check to the king. 
    /// 
    /// Goes like this:
    /// 
    /// \[pin_x, pin_y, pin_diagonal, pin_anti-diagonal\]
    ///
    /// If a piece is simultaneously in two or more pin bitboards, then it 
    /// shall not move at all
    pub pins_bbs: [u64; 4],
    /// Represents a square when in very specific circumstances, the pawn is 
    /// not in a pin for every move except for the en passant capture
    /// 
    /// As far as I am concerned, the only way this square may be not empty 
    /// is when a slider is attacking the king on the en passant rank and 
    /// there is exactly one friendly and one enemy pawn between them
    pub forbidden_en_passant_square: usize,
    pub is_in_double_check: bool,
    /// Bitboards which contain squares that would make a check to the opposite king
    /// based on the piece type
    pub opposite_check_squares_bbs: [u64; 7],
    /// Bitboards for each axis with pieces that, when moved on any other   
    /// axis, would reveal a check to the opposite king
    pub pin_revealer_bbs: [u64; 4]
}

impl AttackCalculator {
    pub fn new(board: &Board) -> Self {
        let mut attack_calculator = Self {
            squares_in_attack_bb: [0; 7],
            check_block_bb: 0,
            pins_bbs: unsafe { std::mem::zeroed() },
            forbidden_en_passant_square: INVALID_SQUARE,
            is_in_double_check: false,
            opposite_check_squares_bbs: unsafe { std::mem::zeroed() },
            pin_revealer_bbs: unsafe { std::mem::zeroed() }
        };

        attack_calculator.calculate_attacks_and_checks(board);
        attack_calculator.calculate_opposite_vulnerabilities(board);

        attack_calculator
    }

    /// Returns true if the friendly king is in check by chess rules
    pub fn in_check(&self) -> bool {
        self.check_block_bb != 0 || self.is_in_double_check
    }

    pub fn print(attack_calculator: &Self) {
        println!("Attacked squares({}): ", attack_calculator.squares_in_attack_bb[0].count_ones());
        board_representation::print_bitboard(attack_calculator.squares_in_attack_bb[0]);
        println!("Squares to block check({}): ", attack_calculator.check_block_bb.count_ones());
        board_representation::print_bitboard(attack_calculator.check_block_bb);

        for index in 0..4 {
            println!("Pinned pieces({}): ", attack_calculator.pins_bbs[index].count_ones());
            board_representation::print_bitboard(attack_calculator.pins_bbs[index]);
            let mut pinned_pieces = attack_calculator.pins_bbs[index];
            if pinned_pieces != 0 {
                println!("Pin line: ");
                while pinned_pieces != 0 {
                    let pinned_piece = bitboards::get_ls1b(pinned_pieces);
                    let pin_line = precomputed_data::SQUARE_DATA.get_file_rank_diagonal_mask(
                        pinned_piece, 
                        index
                    );
                    board_representation::print_bitboard(pin_line);
                    pinned_pieces &= !bitboards::get_bit_from_square(pinned_piece);
                }
            }
        }

        println!("Is in check: {}", attack_calculator.check_block_bb != 0);
        println!("Is in double check: {}", attack_calculator.is_in_double_check);
        println!("Forbidden en passant square: {}", attack_calculator.forbidden_en_passant_square);
        
        // Display opposite check squares by piece type
        println!("Opposite check squares by piece type:");
        let piece_names = ["All", "King", "Pawn", "Knight", "Bishop", "Rook", "Queen"];
        for piece_type in 0..7 {
            println!("  {} check squares({}): ", piece_names[piece_type], 
                    attack_calculator.opposite_check_squares_bbs[piece_type].count_ones());
            board_representation::print_bitboard(attack_calculator.opposite_check_squares_bbs[piece_type]);
        }
        
        // Display pin revealers
        println!("Pin revealers by axis type:");
        let axis_names = ["File", "Rank", "Diagonal", "Anti-diagonal"];
        for axis in 0..4 {
            println!("  {} pin revealers({}): ", axis_names[axis], 
                    attack_calculator.pin_revealer_bbs[axis].count_ones());
            board_representation::print_bitboard(attack_calculator.pin_revealer_bbs[axis]);
        }
    }

    fn add_pin_square_from_bb(&mut self, bitboard: u64, pin_index: usize) {
        debug_assert!(pin_index < 4, "Pin index must be in range 0..4");
        unsafe {
            *self.pins_bbs.get_unchecked_mut(pin_index) |= bitboard;
        }
    }

    fn add_opposite_check_square_bb(&mut self, bb: u64, piece_type: usize) {
        debug_assert!(piece_type < 7, "Piece type must be in range 0..7");
        unsafe {
            *self.opposite_check_squares_bbs.get_unchecked_mut(piece_type) |= bb;
        }
    }
    
    fn add_pin_revealer_square_bb(&mut self, bb: u64, pin_index: usize) {
        debug_assert!(pin_index < 4, "Pin index must be in range 0..4");
        unsafe {
            *self.pin_revealer_bbs.get_unchecked_mut(pin_index) |= bb;
        }
    }

    fn calculate_attacks_and_checks(&mut self, board: &Board) {
        let current_color = board.get_current_color();
        let opposite_color = board.get_opposite_color();
        let king_square = board.get_king_square(current_color);
        let king_bb = board.get_piece_bitboard(current_color, piece::KING);
        let not_king_bb = !king_bb;
        let all_occupied_squares = board.get_all_occupied_squares();
        let all_occupied_squares_without_king = all_occupied_squares & not_king_bb;
        let en_passant_rank_mask = get_en_passant_rank_mask(opposite_color);
        let current_pawn_bb = board.get_piece_bitboard(current_color, piece::PAWN);
        let opposite_pawn_bb = board.get_piece_bitboard(opposite_color, piece::PAWN);

        self.calculate_king(board, opposite_color);
        self.calculate_pawns(opposite_color, king_bb, opposite_pawn_bb);
        self.calculate_knights(board, opposite_color, king_bb);
        self.calculate_sliders(board, opposite_color, all_occupied_squares, !all_occupied_squares, king_square, not_king_bb, all_occupied_squares_without_king, en_passant_rank_mask, current_pawn_bb, opposite_pawn_bb);

        // in result of the calculations, the enemy pieces may technically be pinned 
        // but it doesn't seem to bother anything so we can skip clearing the pins for optimization purposes
    }

    fn calculate_king(&mut self, board: &Board, opposite_color: usize) {
        let attacked_squares_bb = precomputed_data::SQUARE_DATA.get_bb_for_king(
            board.get_king_square(
                opposite_color
            )
        );
        self.squares_in_attack_bb[piece::KING] |=  attacked_squares_bb;
        self.squares_in_attack_bb[0] |= attacked_squares_bb;
    }

    fn calculate_pawns(&mut self, opposite_color: usize, king_bb: u64, pawn_bb: u64) {
        let pawn_attack_shift_offset_1 = get_pawn_attack_shift_offset_1(opposite_color);
        let pawn_attack_shift_offset_2 = get_pawn_attack_shift_offset_2(opposite_color);

        let attacked_squares_1 = bitboards::shift_bb(
            pawn_bb & get_relevant_pawn_attacks_mask_1(
                opposite_color
            ), pawn_attack_shift_offset_1
        );
        let attacked_squares_2 = bitboards::shift_bb(
            pawn_bb & get_relevant_pawn_attacks_mask_2(
                opposite_color
            ), pawn_attack_shift_offset_2
        );

        let attacked_squares_bb = attacked_squares_1 | attacked_squares_2;
        self.squares_in_attack_bb[0] |= attacked_squares_bb;
        self.squares_in_attack_bb[piece::PAWN] |= attacked_squares_bb;

        if self.is_in_double_check {
            return;
        }

        if attacked_squares_1 & king_bb != 0 {
            if self.check_block_bb != 0 {
                self.check_block_bb = 0;
                self.is_in_double_check = true;
                return;
            }
            
            let check_pawn_bit = 
                bitboards::shift_bb(
                    king_bb, 
                    -pawn_attack_shift_offset_1
                );

            self.check_block_bb |= check_pawn_bit;
        }

        if attacked_squares_2 & king_bb != 0 {
            if self.check_block_bb != 0 {
                self.check_block_bb = 0;
                self.is_in_double_check = true;
                return;
            }

            let check_pawn_bit = 
                bitboards::shift_bb(
                    king_bb, 
                    -pawn_attack_shift_offset_2
                );

            self.check_block_bb |= check_pawn_bit;
        }
    }

    fn calculate_knights(&mut self, board: &Board, opposite_color: usize, king_bb: u64) {
        let mut knight_bb = board.get_piece_bitboard(opposite_color, piece::KNIGHT);

        while knight_bb != 0 {
            let start_square = bitboards::get_ls1b(knight_bb);
            let knight_bit = bitboards::get_bit_from_square(start_square);
            knight_bb &= knight_bb - 1;

            let attack_bb = precomputed_data::SQUARE_DATA.get_bb_for_knight(start_square);

            self.squares_in_attack_bb[0] |= attack_bb;
            self.squares_in_attack_bb[piece::KNIGHT] |= attack_bb;

            if self.is_in_double_check { // Friendly king can't block the check so further processing is unnecessary
                continue;
            }

            if attack_bb & king_bb != 0 {
                if self.check_block_bb != 0 {
                    self.check_block_bb = 0;
                    self.is_in_double_check = true;
                    continue;
                }

                self.check_block_bb |= knight_bit;
            }
        }
    }

    fn calculate_sliders(&mut self, board: &Board, opposite_color: usize, all_occ: u64, not_all_occ: u64, king_square: usize, not_king_bb: u64, all_occ_without_king: u64, en_passant_rank_mask: u64, current_pawn_bb: u64, opposite_pawn_bb: u64) {
        let mut rook_bb = board.get_piece_bitboard(opposite_color, piece::ROOK);

        while rook_bb != 0 {
            let start_square = bitboards::get_ls1b(rook_bb);
            let start_bit = bitboards::get_bit_from_square(start_square);
            rook_bb &= rook_bb - 1;

            let attack_bb = precomputed_data::ROOK_MAGIC_LOOKUP_TABLE.get_pseudo_legal_move_bb(
                start_square, 
                all_occ_without_king
            );

            self.squares_in_attack_bb[0] |= attack_bb;
            self.squares_in_attack_bb[piece::ROOK] |= attack_bb;

            if self.is_in_double_check {
                continue;
            }

            let line_to_king_bb = precomputed_data::SQUARE_DATA.get_bb_line(
                start_square, 
                king_square
            );

            let line_to_king_without_slider = line_to_king_bb & !start_bit;
            let line_bb = line_to_king_without_slider & not_king_bb;

            if line_to_king_without_slider & attack_bb == 0 {
                continue; // the queen doesn't attack the king
            }

            if line_bb & not_all_occ == line_bb {
                if self.check_block_bb != 0 {
                    self.check_block_bb = 0; // since now the friendly king is in double check, it's impossible to block
                    self.is_in_double_check = true;
                    continue;
                }

                self.check_block_bb |= line_to_king_bb & not_king_bb;
                continue;
            }

            for direction_index in 0..2 {
                let ray_mask = precomputed_data::SQUARE_DATA.get_file_rank_diagonal_mask(start_square, direction_index);
                let pin_bb = ray_mask & line_bb;
                let all_pinned_pieces = pin_bb & all_occ;
                
                let pin_count = all_pinned_pieces.count_ones();

                if pin_count == 1 {
                    self.add_pin_square_from_bb(all_pinned_pieces, direction_index);
                    break; // we found a pin so we don't need to look for another
                }

                let pinned_current_pawn = pin_bb & current_pawn_bb;
                if ray_mask == en_passant_rank_mask
                    && pin_count == 2
                    && pinned_current_pawn.count_ones() == 1 
                    && (pin_bb & opposite_pawn_bb).count_ones() == 1 {
                        let forbidden_square_bb = pinned_current_pawn;
                        debug_assert!(forbidden_square_bb.count_ones() == 1, "There should be exactly one friendly pawn on the en passant rank");
                        self.forbidden_en_passant_square = bitboards::get_ls1b(forbidden_square_bb);
                    }
            }
        }

        let mut bishop_bb = board.get_piece_bitboard(opposite_color, piece::BISHOP);

        while bishop_bb != 0 {
            let start_square = bitboards::get_ls1b(bishop_bb);
            let start_bit = bitboards::get_bit_from_square(start_square);
            bishop_bb &= bishop_bb - 1;

            let attack_bb = precomputed_data::BISHOP_MAGIC_LOOKUP_TABLE.get_pseudo_legal_move_bb(
                start_square, 
                all_occ_without_king
            );

            self.squares_in_attack_bb[0] |= attack_bb;
            self.squares_in_attack_bb[piece::BISHOP] |= attack_bb;

            if self.is_in_double_check {
                continue;
            }

            let line_to_king_bb = precomputed_data::SQUARE_DATA.get_bb_line(
                start_square, 
                king_square
            );

            let line_to_king_without_slider = line_to_king_bb & !start_bit;
            let line_bb = line_to_king_without_slider & not_king_bb;

            if line_to_king_without_slider & attack_bb == 0 {
                continue; // the queen doesn't attack the king
            }

            if line_bb & not_all_occ == line_bb {
                if self.check_block_bb != 0 {
                    self.check_block_bb = 0; // since now the friendly king is in double check, it's impossible to block
                    self.is_in_double_check = true;
                    continue;
                }

                self.check_block_bb |= line_to_king_bb & not_king_bb;
                continue;
            }

            for direction_index in 2..4 {
                let ray_mask = precomputed_data::SQUARE_DATA.get_file_rank_diagonal_mask(start_square, direction_index);
                let pin_bb = ray_mask & line_bb;
                let all_pinned_pieces = pin_bb & all_occ;
                
                let pin_count = all_pinned_pieces.count_ones();

                if pin_count == 1 {
                    self.add_pin_square_from_bb(all_pinned_pieces, direction_index);
                    break; // we found a pin so we don't need to look for another
                }

                let pinned_current_pawn = pin_bb & current_pawn_bb;
                if ray_mask == en_passant_rank_mask
                    && pin_count == 2
                    && pinned_current_pawn.count_ones() == 1 
                    && (pin_bb & opposite_pawn_bb).count_ones() == 1 {
                        let forbidden_square_bb = pinned_current_pawn;
                        debug_assert!(forbidden_square_bb.count_ones() == 1, "There should be exactly one friendly pawn on the en passant rank");
                        self.forbidden_en_passant_square = bitboards::get_ls1b(forbidden_square_bb);
                    }
            }
        }

        let mut queen_bb = board.get_piece_bitboard(opposite_color, piece::QUEEN);

        while queen_bb != 0 {
            let start_square = bitboards::get_ls1b(queen_bb);
            let start_bit = bitboards::get_bit_from_square(start_square);
            queen_bb &= queen_bb - 1;

            let attack_bb = 
                precomputed_data::ROOK_MAGIC_LOOKUP_TABLE.get_pseudo_legal_move_bb(
                    start_square, 
                    all_occ_without_king
                ) | precomputed_data::BISHOP_MAGIC_LOOKUP_TABLE.get_pseudo_legal_move_bb(
                    start_square, 
                    all_occ_without_king
            );
            
            self.squares_in_attack_bb[0] |= attack_bb;
            self.squares_in_attack_bb[piece::QUEEN] |= attack_bb;

            if self.is_in_double_check {
                continue;
            }

            let line_to_king_bb = precomputed_data::SQUARE_DATA.get_bb_line(
                start_square, 
                king_square
            );

            let line_to_king_without_slider = line_to_king_bb & !start_bit;
            let line_bb = line_to_king_without_slider & not_king_bb;

            if line_to_king_without_slider & attack_bb == 0 {
                continue; // the queen doesn't attack the king
            }

            if line_bb & not_all_occ == line_bb {
                if self.check_block_bb != 0 {
                    self.check_block_bb = 0; // since now the friendly king is in double check, it's impossible to block
                    self.is_in_double_check = true;
                    continue;
                }

                self.check_block_bb |= line_to_king_bb & not_king_bb;
                continue;
            }

            for direction_index in 0..4 {
                let ray_mask = precomputed_data::SQUARE_DATA.get_file_rank_diagonal_mask(start_square, direction_index);
                let pin_bb = ray_mask & line_bb;
                let all_pinned_pieces = pin_bb & all_occ;
                
                let pin_count = all_pinned_pieces.count_ones();

                if pin_count == 1 {
                    self.add_pin_square_from_bb(all_pinned_pieces, direction_index);
                    break; // we found a pin so we don't need to look for another
                }

                let pinned_current_pawn = pin_bb & current_pawn_bb;
                if ray_mask == en_passant_rank_mask
                    && pin_count == 2
                    && pinned_current_pawn.count_ones() == 1 
                    && (pin_bb & opposite_pawn_bb).count_ones() == 1 {
                        let forbidden_square_bb = pinned_current_pawn;
                        debug_assert!(forbidden_square_bb.count_ones() == 1, "There should be exactly one friendly pawn on the en passant rank");
                        self.forbidden_en_passant_square = bitboards::get_ls1b(forbidden_square_bb);
                    }
            }
        }
    }

    /// Calculates the squares where the current player can go to give a check
    /// 
    /// Allows to quickly determine whether a move is a check (even more sophisticated checks like 
    /// discovered checks or double checks)
    fn calculate_opposite_vulnerabilities(&mut self, board: &Board) {
        let current_color = board.get_current_color();
        let opposite_color = board.get_opposite_color();
        let opposite_king_square = board.get_king_square(opposite_color);
        let opposite_king_bb = board.get_piece_bitboard(opposite_color, piece::KING);
        let all_occ = board.get_all_occupied_squares();

        // no king calculations since king can't give checks

        let pawn_attack_bb = bitboards::shift_bb(opposite_king_bb, get_pawn_attack_shift_offset_1(opposite_color)) | bitboards::shift_bb(opposite_king_bb, get_pawn_attack_shift_offset_2(opposite_color));
        self.add_opposite_check_square_bb(pawn_attack_bb, piece::PAWN);

        let knight_attack_bb = precomputed_data::SQUARE_DATA.get_bb_for_knight(opposite_king_square);
        self.add_opposite_check_square_bb(knight_attack_bb, piece::KNIGHT);

        let rook_attack_bb = precomputed_data::ROOK_MAGIC_LOOKUP_TABLE.get_pseudo_legal_move_bb(opposite_king_square, all_occ);
        self.add_opposite_check_square_bb(rook_attack_bb, piece::ROOK);

        let bishop_attack_bb = precomputed_data::BISHOP_MAGIC_LOOKUP_TABLE.get_pseudo_legal_move_bb(opposite_king_square, all_occ);
        self.add_opposite_check_square_bb(bishop_attack_bb, piece::BISHOP);

        self.add_opposite_check_square_bb(rook_attack_bb | bishop_attack_bb, piece::QUEEN);

        // Calculate pin revealers - pieces that when moved would reveal a check to the opposite king
        self.calculate_pin_revealers(board, opposite_king_square, current_color);
    }

    fn calculate_pin_revealers(&mut self, board: &Board, opposite_king_square: usize, current_color: usize) {
        let all_occ = board.get_all_occupied_squares();
        let opposite_king_bb = bitboards::get_bit_from_square(opposite_king_square);
        let not_opposite_king_bb = !opposite_king_bb;

        let queen_bb = board.get_piece_bitboard(current_color, piece::QUEEN);
        
        // Check for potential rook-like pin revealers (horizontal and vertical)
        let mut rooks_and_queens = board.get_piece_bitboard(current_color, piece::ROOK) | 
                                  queen_bb;
        
        while rooks_and_queens != 0 {
            let slider_square = bitboards::get_ls1b(rooks_and_queens);
            rooks_and_queens &= rooks_and_queens - 1;
            
            // Get the line between the slider and the opposite king
            let line_bb = precomputed_data::SQUARE_DATA.get_bb_line(slider_square, opposite_king_square);
            
            // If there's no line (not on same rank/file), continue
            if line_bb == 0 {
                continue;
            }
            
            // Get all pieces on this line excluding the slider and king
            let pieces_on_line = line_bb & all_occ & 
                                !bitboards::get_bit_from_square(slider_square) & 
                                not_opposite_king_bb;
            
            // If there's exactly one piece between them, it's a potential pin revealer
            if pieces_on_line.count_ones() == 1 {
                // Determine if it's a file or rank pin
                for direction_index in 0..2 {
                    let ray_mask = precomputed_data::SQUARE_DATA.get_file_rank_diagonal_mask(slider_square, direction_index);
                    if ray_mask & opposite_king_bb != 0 {
                        self.add_pin_revealer_square_bb(pieces_on_line, direction_index);
                        break;
                    }
                }
            }
        }
        
        // Check for potential bishop-like pin revealers (diagonal)
        let mut bishops_and_queens = board.get_piece_bitboard(current_color, piece::BISHOP) | 
                                   queen_bb;
        
        while bishops_and_queens != 0 {
            let slider_square = bitboards::get_ls1b(bishops_and_queens);
            bishops_and_queens &= bishops_and_queens - 1;
            
            // Get the line between the slider and the opposite king
            let line_bb = precomputed_data::SQUARE_DATA.get_bb_line(slider_square, opposite_king_square);
            
            // If there's no line (not on same diagonal), continue
            if line_bb == 0 {
                continue;
            }
            
            // Get all pieces on this line excluding the slider and king
            let pieces_on_line = line_bb & all_occ & 
                                !bitboards::get_bit_from_square(slider_square) & 
                                not_opposite_king_bb;
            
            // If there's exactly one piece between them, it's a potential pin revealer
            if pieces_on_line.count_ones() == 1 {
                // Determine if it's a diagonal or anti-diagonal pin
                for direction_index in 2..4 {
                    let ray_mask = precomputed_data::SQUARE_DATA.get_file_rank_diagonal_mask(slider_square, direction_index);
                    if ray_mask & opposite_king_bb != 0 {
                        self.add_pin_revealer_square_bb(pieces_on_line, direction_index);
                        break;
                    }
                }
            }
        }
    }
}