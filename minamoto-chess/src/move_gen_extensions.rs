use minamoto_chess_core::r#move::Move;

pub fn get_legal_moves_from_square(
    all_legal_moves: &[Move],
    square: usize
) -> Vec<Move> {
    all_legal_moves.iter()
        .filter(|m| m.start_square == square)
        .cloned()
        .collect()
}