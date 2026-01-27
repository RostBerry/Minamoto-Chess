use std::time::Duration;

use crate::perft::perft_node::PerftNode;

pub struct PerftResult {
    pub total_nodes: usize,
    pub nodes_by_move: Vec<PerftNode>,
    pub duration: Duration,
}