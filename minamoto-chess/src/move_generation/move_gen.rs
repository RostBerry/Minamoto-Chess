use once_cell::sync::Lazy;

use crate::{bitboards, board::{self, Board}, castling, r#move::{Move, MoveType}, piece, precomputed_data::{self, magic_lookup_table::MagicLookupTable}};

use super::attack_calculator::AttackCalculator;

// constants
/// Represents the theoretical maximum number of 
/// available moves in a single chess position. 
/// The number was eyeballed
pub const MAX_MOVES_PER_POS: usize = 150;

const KING_SIDE_CASTLING_MASK: [u64; 2] = [bitboards::WHITE_KING_SIDE_CASTLING_MASK, bitboards::BLACK_KING_SIDE_CASTLING_MASK];
fn get_king_side_castling_mask(color: usize) -> u64 {
    debug_assert!(color < 2, "Color index out of bounds");
    unsafe { *KING_SIDE_CASTLING_MASK.get_unchecked(color) }
}
const QUEEN_SIDE_CASTLING_MASK: [u64; 2] = [bitboards::WHITE_QUEEN_SIDE_CASTLING_MASK, bitboards::BLACK_QUEEN_SIDE_CASTLING_MASK];
fn get_queen_side_castling_mask(color: usize) -> u64 {
    debug_assert!(color < 2, "Color index out of bounds");
    unsafe { *QUEEN_SIDE_CASTLING_MASK.get_unchecked(color) }
}
const QUEEN_SIDE_CASTLING_PATH_MASK: [u64; 2] = [bitboards::WHITE_QUEEN_SIDE_CASTLING_KING_PATH_MASK, bitboards::BLACK_QUEEN_SIDE_CASTLING_KING_PATH_MASK];
fn get_queen_side_castling_path_mask(color: usize) -> u64 {
    debug_assert!(color < 2, "Color index out of bounds");
    unsafe { *QUEEN_SIDE_CASTLING_PATH_MASK.get_unchecked(color) }
}

const PAWN_SHIFT_AMOUNT: [i8; 2] = [8, -8];
fn get_pawn_shift_amount(color: usize) -> i8 {
    debug_assert!(color < 2, "Color index out of bounds");
    unsafe { *PAWN_SHIFT_AMOUNT.get_unchecked(color) }
}
const PAWN_PROMOTION_RANK: [u64; 2] = [bitboards::EIGHTH_RANK_MASK, bitboards::FIRST_RANK_MASK];
fn get_pawn_promotion_rank(color: usize) -> u64 {
    debug_assert!(color < 2, "Color index out of bounds");
    unsafe { *PAWN_PROMOTION_RANK.get_unchecked(color) }
}
const PAWN_START_RANK: [u64; 2] = [bitboards::SECOND_RANK_MASK, bitboards::SEVENTH_RANK_MASK];
fn get_pawn_start_rank(color: usize) -> u64 {
    debug_assert!(color < 2, "Color index out of bounds");
    unsafe { *PAWN_START_RANK.get_unchecked(color) }
}

fn add_move(moves: &mut Vec<Move>, move_: Move) {
    debug_assert!(moves.len() < MAX_MOVES_PER_POS, "Moves vector is full");
    moves.push(move_);
}

pub fn print(moves: &[Move]) {
    println!("All moves({}):", moves.len());
    for mov in moves {
        print!("{} ", mov);
    }
    println!();
}

fn add_promotion_moves(
    moves: &mut Vec<Move>,
    start_square: usize, 
    target_square: usize, 
    capture_square: usize,
) {
    add_move(
        moves,
        Move::new(
            start_square, 
            target_square, 
            capture_square, 
            MoveType::PromotionQueen,
        )
    );

    add_move(
        moves,
        Move::new(
            start_square, 
            target_square, 
            capture_square, 
            MoveType::PromotionKnight,
        )
    );

    add_move(
        moves,
        Move::new(
            start_square, 
            target_square, 
            capture_square, 
            MoveType::PromotionRook,
        )
    );

    add_move(
        moves,
        Move::new(
            start_square, 
            target_square, 
            capture_square, 
            MoveType::PromotionBishop,
        )
    );
}

pub fn generate_moves(moves: &mut Vec<Move>, board: &Board, attack_calculator: &AttackCalculator) {
    let current_color = board.get_current_color();
    let king_square = board.get_king_square(current_color);
    let attacked_squares_bb = attack_calculator.squares_in_attack_bb[0]; 
    let all_occ = board.get_all_occupied_squares();
    let not_all_current_occ = !board.get_all_occupied_squares_for_color(current_color);

    generate_king(moves, board, current_color, king_square, attacked_squares_bb, all_occ, not_all_current_occ, attack_calculator.in_check());
    if attack_calculator.is_in_double_check {
        return; // No other moves are possible
    }

    let check_block_bb = attack_calculator.check_block_bb;
    let is_single_check = check_block_bb != 0;
    let opposite_color = board.get_opposite_color();
    let all_opposite_occ = board.get_all_occupied_squares_for_color(opposite_color);
    let pin_bbs = attack_calculator.pins_bbs;

    generate_pawns(moves, board, &attack_calculator, current_color, all_occ, all_opposite_occ, check_block_bb, &pin_bbs, is_single_check);

    let pin_bb_1 = unsafe { *pin_bbs.get_unchecked(0) };
    let pin_bb_2 = unsafe { *pin_bbs.get_unchecked(1) };
    let pin_bb_3 = unsafe { *pin_bbs.get_unchecked(2) };
    let pin_bb_4 = unsafe { *pin_bbs.get_unchecked(3) };

    let first_second_pins = pin_bb_1 | pin_bb_2;
    let third_fourth_pins = pin_bb_3 | pin_bb_4;

    generate_knights(moves, board, current_color, not_all_current_occ, check_block_bb, first_second_pins | third_fourth_pins);

    generate_sliders(moves, board, piece::ROOK, third_fourth_pins, pin_bb_1, pin_bb_2, 0, 1, &precomputed_data::ROOK_MAGIC_LOOKUP_TABLE, current_color, is_single_check, check_block_bb, all_occ, not_all_current_occ);
    generate_sliders(moves, board, piece::BISHOP, first_second_pins, pin_bb_3, pin_bb_4, 2, 3, &precomputed_data::BISHOP_MAGIC_LOOKUP_TABLE, current_color, is_single_check, check_block_bb, all_occ, not_all_current_occ);
    generate_queens(moves, board, current_color, all_occ, not_all_current_occ, is_single_check, check_block_bb, &pin_bbs);
}

fn generate_king(moves: &mut Vec<Move>, board: &Board, current_color: usize, king_square: usize, attacked_squares_bb: u64, all_occ: u64, not_all_current_occ: u64, is_check: bool) {
    let mut pseudo_moves_bb = precomputed_data::SQUARE_DATA.get_bb_for_king(king_square) 
    & !attacked_squares_bb // The king cannot move to a square that is attacked
    & not_all_current_occ; 
    // The king cannot move to a square that is occupied by a friendly piece

    while pseudo_moves_bb != 0 {
        let target_square = bitboards::get_ls1b(pseudo_moves_bb);
        pseudo_moves_bb &= pseudo_moves_bb - 1;

        add_move(
            moves,
            Move::new(
                king_square, 
                target_square, 
                target_square, 
                MoveType::Regular,
            )
        );
    }

    if is_check {
        return; // The king cannot castle if it is in check
    }

    let castling_blocker_bb = all_occ | attacked_squares_bb;
    let castling_state = board.get_castling_state(current_color);

    if castling::can_king_side(castling_state) {
        let king_side_castling_mask = get_king_side_castling_mask(current_color);

        if (king_side_castling_mask & castling_blocker_bb) == 0 {
            let king_side_castling_square = board::get_king_side_square(current_color);
            add_move(
                moves,
                Move::new(
                    king_square, 
                    king_side_castling_square, 
                    king_side_castling_square, 
                    MoveType::CastlingKingSide,
                )
            );
        }
    }

    if castling::can_queen_side(castling_state) {
        let queen_side_castling_mask = get_queen_side_castling_mask(current_color);

        let queen_side_castling_king_path_mask = get_queen_side_castling_path_mask(current_color);

        if (queen_side_castling_mask & all_occ) == 0 
        && (queen_side_castling_king_path_mask & attacked_squares_bb) == 0 {
            let queen_side_castling_square = board::get_queen_side_square(current_color);
            add_move(
                moves,
                Move::new(
                    king_square, 
                    queen_side_castling_square, 
                    queen_side_castling_square, 
                    MoveType::CastlingQueenSide,
                )
            );
        }
    }
}

fn generate_pawns(moves: &mut Vec<Move>, board: &Board, attack_calculator: &AttackCalculator, current_color: usize, all_occ: u64, all_opposite_occ: u64, check_block_bb: u64, pin_bbs: &[u64; 4], is_check: bool) {
    let not_all_occ = !all_occ;
    
    let shift_amount = get_pawn_shift_amount(current_color);
    let neg_shift_amount = -shift_amount;
    let two_squares_shift_amount = shift_amount * 2;
    let shift_add_1 = shift_amount + 1;
    let shift_sub_1 = shift_amount - 1;

    let promotion_rank_bb = get_pawn_promotion_rank(current_color);
    let start_rank_bb = get_pawn_start_rank(current_color);

    let forbidden_en_passant_square = attack_calculator.forbidden_en_passant_square;
    let is_en_passant_possible = board.is_en_passant_possible();
    // Only compute these values if en-passant is possible
    let (en_passant_square, en_passant_capture, en_passant_square_bb) = if is_en_passant_possible {
        let square = board.en_passant_pawn_square();
        let capture = bitboards::get_bit_from_square(board.en_passant_capture_square());
        let square_bb = bitboards::get_bit_from_square(square);
        (square, capture, square_bb)
    } else {
        (0, 0, 0)
    };

    let en_passant_mask_or_check = en_passant_capture | check_block_bb;
    let en_passant_mask_or_all = all_opposite_occ | en_passant_capture;
    let en_passant_on_check = en_passant_square_bb & check_block_bb != 0;

    let pawn_bb = board.get_piece_bitboard(current_color, piece::PAWN);

    // We can prune the move generation for pawns pinned on any axes that are not the pawn's movement direction
    let move_forward_irrelevant_pins = unsafe {
        *pin_bbs.get_unchecked(1) 
        | *pin_bbs.get_unchecked(2) 
        | *pin_bbs.get_unchecked(3)
    };

    let mut pawn_one_square_bb = bitboards::shift_bb(
        pawn_bb & !move_forward_irrelevant_pins, 
        shift_amount
    ) & not_all_occ; // Pawns can only move one square forward if the square is not occupied
    
    let pawn_one_square_bb_copy = pawn_one_square_bb; 
    // copying before the check pruning becuase if a pawn can't move one square to block the 
    // check, it doesn't mean it can't move two squares

    if is_check {
        pawn_one_square_bb &= check_block_bb;
    } // Pawns must block the check if there is a check

    while pawn_one_square_bb != 0 {
        let target_square = bitboards::get_ls1b(pawn_one_square_bb);
        let target_square_bb = bitboards::get_bit_from_square(target_square);
        pawn_one_square_bb &= pawn_one_square_bb - 1;

        let start_square = (target_square as i8 - shift_amount) as usize;

        if (promotion_rank_bb & target_square_bb) == 0 {
            add_move(
                moves,
                Move::new(
                    start_square, 
                    target_square, 
                    target_square, 
                    MoveType::Regular,
                )
            );
            continue;
        }

        add_promotion_moves(
            moves,
            start_square, 
            target_square, 
            target_square
        );
    }

    let pawns_start_squares_bb = bitboards::shift_bb( 
        // Pawns can move two squares forward only if they can move one square forward
        pawn_one_square_bb_copy, 
        neg_shift_amount
    ) & start_rank_bb; // Pawns can move two squares forward only if they are on the starting rank

    let mut pawn_two_squares_bb = bitboards::shift_bb(
        pawns_start_squares_bb, 
        two_squares_shift_amount
    ) & not_all_occ; // Pawns can only move two squares forward if the squares are not occupied

    if is_check {
        pawn_two_squares_bb &= check_block_bb;
    } // Pawns must block the check if there is a check

    while pawn_two_squares_bb != 0 {
        let target_square = bitboards::get_ls1b(pawn_two_squares_bb);
        pawn_two_squares_bb &= pawn_two_squares_bb - 1;

        let start_square = (target_square as i8 - two_squares_shift_amount) as usize;

        add_move(
            moves,
            Move::new(
                start_square, 
                target_square, 
                target_square, 
                MoveType::PawnDoubleMove
            )
        );
    }

    let diagonal_irrelevant_pins = unsafe {
        *pin_bbs.get_unchecked(0)
        | *pin_bbs.get_unchecked(1)
        | *pin_bbs.get_unchecked(3)
    };
    let anti_diagonal_irrelevant_pins = unsafe {
        *pin_bbs.get_unchecked(0)
        | *pin_bbs.get_unchecked(1)
        | *pin_bbs.get_unchecked(2)
    };

    let move_west_irrelevant_pins = match current_color {
        piece::WHITE => diagonal_irrelevant_pins,
        piece::BLACK => anti_diagonal_irrelevant_pins,
        _ => panic!("Invalid color: {}", current_color),
    };

    let mut capture_west_bb = bitboards::shift_bb(
        pawn_bb & bitboards::NOT_FIRST_FILE_MASK & !move_west_irrelevant_pins, 
        shift_add_1
    ) & en_passant_mask_or_all; // Pawns can capture en passant 

    if is_check {
        capture_west_bb &= en_passant_mask_or_check;
    } // Pawns must block the check if there is a check

    while capture_west_bb != 0 {
        let target_square = bitboards::get_ls1b(capture_west_bb);
        let target_square_bb = bitboards::get_bit_from_square(target_square);
        capture_west_bb &= capture_west_bb - 1;

        let start_square = (target_square as i8 - shift_add_1) as usize;

        let is_en_passant = target_square_bb & en_passant_capture != 0;

        if (forbidden_en_passant_square == start_square && is_en_passant) 
        || (is_check && !en_passant_on_check && is_en_passant) {
            continue;
        }

        let capture_square = if is_en_passant {
            en_passant_square
        } else {
            target_square
        };

        if (promotion_rank_bb & target_square_bb) == 0 {
            add_move(
                moves,
                Move::new(
                    start_square, 
                    target_square, 
                    capture_square, 
                    MoveType::Regular
                )
            );
            continue;
        }

        add_promotion_moves(
            moves,
            start_square, 
            target_square, 
            capture_square
        );
    }

    let capture_east_irrelevant_pins = match current_color {
        piece::WHITE => anti_diagonal_irrelevant_pins,
        piece::BLACK => diagonal_irrelevant_pins,
        _ => panic!("Invalid color: {}", current_color),
    };

    let mut capture_east_bb = bitboards::shift_bb(
        pawn_bb & bitboards::NOT_EIGHTH_FILE_MASK & !capture_east_irrelevant_pins, 
        shift_sub_1
    ) & en_passant_mask_or_all; // Pawns can capture en passant 

    if is_check {
        capture_east_bb &= en_passant_mask_or_check;
    } // Pawns must block the check if there is a check

    while capture_east_bb != 0 {
        let target_square = bitboards::get_ls1b(capture_east_bb);
        let target_square_bb = bitboards::get_bit_from_square(target_square);
        capture_east_bb &= capture_east_bb - 1;

        let start_square = (target_square as i8 - shift_sub_1) as usize;

        let is_en_passant = target_square_bb & en_passant_capture != 0;

        if (forbidden_en_passant_square == start_square && is_en_passant)  // The move is en passant but it is forbidden
        || (is_check && !en_passant_on_check && is_en_passant) { // The move is en passant but it doesn't block the check
            continue;
        }

        let capture_square = if is_en_passant {
            en_passant_square
        } else {
            target_square
        };

        if (promotion_rank_bb & target_square_bb) == 0 {
            add_move(
                moves,
                Move::new(
                    start_square, 
                    target_square, 
                    capture_square, 
                    MoveType::Regular
                )
            );
            continue;
        }

        add_promotion_moves(
            moves,
            start_square, 
            target_square, 
            capture_square
        );
    }
}

fn generate_knights(moves: &mut Vec<Move>, board: &Board, current_color: usize, not_all_current_occ: u64, check_block_bb: u64, pins_bb: u64) {
    let mut knight_bb = board.get_piece_bitboard(current_color, piece::KNIGHT);

    knight_bb &= !pins_bb; // We can prune the move generation for the pinned knights 
    // because it's impossible to move for them if they are pinned

    while knight_bb != 0 {
        let start_square = bitboards::get_ls1b(knight_bb);
        knight_bb &= knight_bb - 1;

        let mut pseudo_moves_bb = precomputed_data::SQUARE_DATA
            .get_bb_for_knight(start_square) 
            & not_all_current_occ;

        if check_block_bb != 0 {
            pseudo_moves_bb &= check_block_bb;
        } // Knights must block the check if there is a check

        while pseudo_moves_bb != 0 {
            let target_square = bitboards::get_ls1b(pseudo_moves_bb);
            pseudo_moves_bb &= pseudo_moves_bb - 1;

            add_move(
                moves,
                Move::new(
                    start_square, 
                    target_square, 
                    target_square, 
                    MoveType::Regular
                )
            );
        }
    }
}

fn generate_sliders(moves: &mut Vec<Move>, board: &Board, slider_type: usize, irrelevant_pins_bb: u64, relevant_pin_bb_1: u64, relevant_pin_bb_2: u64, relevant_pin_index_1: usize, relevant_pin_index_2: usize, magic_lookup_table: &Lazy<Box<MagicLookupTable>>, current_color: usize, is_check: bool, check_block_bb: u64, all_occ: u64, not_all_occ_current_color: u64) {    
    let mut slider_bb = board.get_piece_bitboard(current_color, slider_type) & !irrelevant_pins_bb;

    while slider_bb != 0 {
        let start_square = bitboards::get_ls1b(slider_bb);
        let start_square_bb = bitboards::get_bit_from_square(start_square);
        slider_bb &= slider_bb - 1;

        let mut pseudo_moves_bb = magic_lookup_table
            .get_pseudo_legal_move_bb(
                start_square, 
                all_occ
            ) & not_all_occ_current_color;

        if is_check {
            pseudo_moves_bb &= check_block_bb;
        } // Sliders must block the check if there is a check

        if relevant_pin_bb_1 & start_square_bb != 0 {
            pseudo_moves_bb &= precomputed_data::SQUARE_DATA.get_file_rank_diagonal_mask(start_square, relevant_pin_index_1);
        }
        if relevant_pin_bb_2 & start_square_bb != 0 {
            pseudo_moves_bb &= precomputed_data::SQUARE_DATA.get_file_rank_diagonal_mask(start_square, relevant_pin_index_2);
        }

        while pseudo_moves_bb != 0 {
            let target_square = bitboards::get_ls1b(pseudo_moves_bb);
            pseudo_moves_bb &= pseudo_moves_bb - 1;

            add_move(
                moves,
                Move::new(
                    start_square, 
                    target_square, 
                    target_square, 
                    MoveType::Regular
                )
            );
        }
    }
}

fn generate_queens(moves: &mut Vec<Move>, board: &Board, current_color: usize, all_occ: u64, not_all_occ_current_color: u64, is_single_check: bool, squares_to_block_check_bb: u64, pin_bbs: &[u64; 4]) {
    let mut queen_bb = board.get_piece_bitboard(current_color, piece::QUEEN);

    while queen_bb != 0 {
        let start_square = bitboards::get_ls1b(queen_bb);
        let start_square_bb = bitboards::get_bit_from_square(start_square);
        queen_bb &= queen_bb - 1;

        // Get both rook and bishop moves for the queen
        let mut pseudo_moves_bb = (
        precomputed_data::ROOK_MAGIC_LOOKUP_TABLE
            .get_pseudo_legal_move_bb(start_square, all_occ) |
        precomputed_data::BISHOP_MAGIC_LOOKUP_TABLE
            .get_pseudo_legal_move_bb(start_square, all_occ)
        ) & not_all_occ_current_color;

        if is_single_check {
            pseudo_moves_bb &= squares_to_block_check_bb;
        }

        for direction_index in 0..4 {
            let pin_bb = unsafe { *pin_bbs.get_unchecked(direction_index) };
            if (pin_bb & start_square_bb) != 0 {
                pseudo_moves_bb &= precomputed_data::SQUARE_DATA.get_file_rank_diagonal_mask(start_square, direction_index);
                break;
            }
        }

        while pseudo_moves_bb != 0 {
            let target_square = bitboards::get_ls1b(pseudo_moves_bb);
            pseudo_moves_bb &= pseudo_moves_bb - 1;

            add_move(
                moves,
                Move::new(
                    start_square, 
                    target_square, 
                    target_square, 
                    MoveType::Regular
                )
            );
        }
    }
}

pub fn filter_loud_moves(legal_moves: &mut Vec<Move>, filtered_moves: &mut Vec<Move>, attack_calc: &AttackCalculator, board: &Board) {
    let move_count = legal_moves.len();
    for i in 0..move_count {
        let mov = unsafe {
            std::ptr::read(legal_moves.as_ptr().add(i))
        };
        let (_, piece_type) = board.get_piece_on_square(mov.start_square);

        let target_square_bb = bitboards::get_bit_from_square(mov.target_square);
        let start_square_bb = bitboards::get_bit_from_square(mov.start_square);

        if target_square_bb & unsafe { std::ptr::read(attack_calc.opposite_check_squares_bbs.as_ptr().add(piece_type)) } != 0 {
            filtered_moves.push(mov);
            continue;
        }

        let (_, capture_square_piece_type) = board.get_piece_on_square(mov.capture_square);
        if capture_square_piece_type != piece::NONE {
            filtered_moves.push(mov);
            continue;
        }

        for direction_index in 0..4 {
            let pin_revealer_bb = unsafe { std::ptr::read(attack_calc.pin_revealer_bbs.as_ptr().add(direction_index)) };
            let line_mask = precomputed_data::SQUARE_DATA.get_file_rank_diagonal_mask(mov.start_square, direction_index);

            if (pin_revealer_bb & start_square_bb) != 0 && (line_mask & target_square_bb) != 0 {
                filtered_moves.push(mov);
                break;
            }
        }
    }
}