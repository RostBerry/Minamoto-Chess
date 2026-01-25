use crate::r#move::UciMove;

/// Represents a move and the number of its children nodes
pub struct PerftNode {
    mov: UciMove,
    nodes: usize,
}

impl PerftNode {
    pub fn new(mov: UciMove, nodes: usize) -> PerftNode {
        PerftNode {
            mov,
            nodes,
        }
    }

    pub fn get_move(&self) -> &UciMove {
        &self.mov
    }

    pub fn get_node_count(&self) -> usize {
        self.nodes
    }
}