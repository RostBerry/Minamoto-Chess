use minamoto_chess_core::r#move::Move;

/// Represents a move and the number of its children nodes
pub struct PerftNode {
    mov: Move,
    nodes: usize,
}

impl PerftNode {
    pub fn new(mov: Move, nodes: usize) -> PerftNode {
        PerftNode {
            mov,
            nodes,
        }
    }

    pub fn get_move(&self) -> &Move {
        &self.mov
    }

    pub fn get_node_count(&self) -> usize {
        self.nodes
    }
}