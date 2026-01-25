use std::time::Instant;

use crate::{board::Board, config, r#move::Move, move_generation::{attack_calculator::AttackCalculator, move_gen}};

pub mod perft_node;

pub fn run_perft(depth: u8, board: &mut Board) -> u64 {
    let start = Instant::now();

    
    let mut move_buffer: Vec<Vec<Move>> = (0..depth)
        .map(|_| Vec::with_capacity(move_gen::MAX_MOVES_PER_POS))
        .collect();

    let result = count_nodes(depth, board, &mut move_buffer);
    let duration = start.elapsed();

    if config::DO_TERMINAL_OUTPUT {
        println!("Perft result: {}", result);
        println!("Time taken: {:?}", duration);
    }
    
    result
}

fn count_nodes(depth: u8, board: &mut Board, move_buffer: &mut [Vec<Move>]) -> u64 {
    debug_assert!(depth > 0, "Depth must be greater than 0");
    
    let (current_moves, remaining_buffer) = move_buffer.split_at_mut(1);
    let mut current_moves = unsafe { current_moves.get_unchecked_mut(0) };
    current_moves.clear();
    let attack_calc = AttackCalculator::new(board);
    move_gen::generate_moves(&mut current_moves, board, &attack_calc);

    if depth == 1 {
        return current_moves.len() as u64;
    }

    let do_output = depth == config::PERFT_DEPTH && config::DO_TERMINAL_OUTPUT;

    if depth == 2 {
        let mut nodes = 0;
        let (child_moves, _) = remaining_buffer.split_at_mut(1);
        let mut child_moves = unsafe { child_moves.get_unchecked_mut(0) };

        for mov in current_moves.drain(..) {
            if do_output {
                print!("{}: ", mov);
            }
            let move_record = board.make_move(mov);
            child_moves.clear();
            let attack_calc = AttackCalculator::new(board);
            move_gen::generate_moves(&mut child_moves, board, &attack_calc);
            board.undo_move(move_record);
            let child_nodes = child_moves.len() as u64;
            nodes += child_nodes;

            if do_output {
                println!("{}", child_nodes);
            }
        }
        return nodes;
    }

    let mut nodes = 0;

    for mov in current_moves.drain(..) {
        if do_output {
            print!("{}: ", mov);
        }
        let move_record = board.make_move(mov);
        let child_nodes = count_nodes(depth - 1, board, remaining_buffer);
        board.undo_move(move_record);
        nodes += child_nodes;

        if do_output {
            println!("{}", child_nodes);
        }
    }

    nodes
}