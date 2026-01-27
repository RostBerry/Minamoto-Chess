use std::time::Instant;

use minamoto_chess_core::{board::Board, r#move::Move, move_generation::{attack_calculator::AttackCalculator, move_gen}};

use crate::{perft::{perft_node::PerftNode, perft_result::PerftResult}, uci_move::UciMove};

pub mod perft_node;
pub mod perft_result;

pub fn run_perft(depth: u8, board: &mut Board) -> PerftResult {
    let start = Instant::now();

    let mut main_nodes: Vec<PerftNode> = Vec::with_capacity(move_gen::MAX_MOVES_PER_POS);

    
    let mut move_buffer: Vec<Vec<Move>> = (0..depth)
        .map(|_| Vec::with_capacity(move_gen::MAX_MOVES_PER_POS))
        .collect();

    let (root_moves, remaining_buffer) = move_buffer.split_at_mut(1);
    let root_moves = unsafe { root_moves.get_unchecked_mut(0) };
    root_moves.clear();
    
    let attack_calc = AttackCalculator::new(board);
    move_gen::generate_moves(root_moves, board, &attack_calc);
    
    let mut total_nodes = 0u64;
    
    for mov in root_moves.drain(..) {
        let move_record = board.make_move(mov);
        let nodes = if depth == 1 {
            1
        } else {
            count_nodes(depth - 1, board, remaining_buffer)
        };
        board.undo_move(move_record);
        
        let uci_move = UciMove::from_move(mov);
        main_nodes.push(PerftNode::new(uci_move, nodes as usize));
        total_nodes += nodes;
    }
    
    let duration = start.elapsed();
    
    PerftResult {
        total_nodes: total_nodes as usize,
        nodes_by_move: main_nodes,
        duration: duration,
    }
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

    if depth == 2 {
        let mut nodes = 0;
        let (child_moves, _) = remaining_buffer.split_at_mut(1);
        let mut child_moves = unsafe { child_moves.get_unchecked_mut(0) };

        for mov in current_moves.drain(..) {
            let move_record = board.make_move(mov);
            child_moves.clear();
            let attack_calc = AttackCalculator::new(board);
            move_gen::generate_moves(&mut child_moves, board, &attack_calc);
            board.undo_move(move_record);
            let child_nodes = child_moves.len() as u64;
            nodes += child_nodes;
        }
        return nodes;
    }

    let mut nodes = 0;

    for mov in current_moves.drain(..) {
        let move_record = board.make_move(mov);
        let child_nodes = count_nodes(depth - 1, board, remaining_buffer);
        board.undo_move(move_record);
        nodes += child_nodes;
    }

    nodes
}