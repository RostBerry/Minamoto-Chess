use minamoto_chess_core::{bitboards, move_generation::attack_calculator::AttackCalculator, precomputed_data};

use crate::board_representation;

pub fn print_attack_calculator(attack_calculator: &AttackCalculator) {
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