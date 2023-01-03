use super::*;
use std::collections::VecDeque;
pub struct VisualQueue {
    pub nodes: VecDeque<VisualNode>,
    pub persistent_nodes: Vec<VisualNode>,
}
