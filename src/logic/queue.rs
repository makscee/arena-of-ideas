use std::collections::VecDeque;

use super::*;

pub struct LogicQueue {
    pub nodes: VecDeque<LogicNode>,
}

impl LogicQueue {
    pub fn new() -> Self {
        Self { nodes: default() }
    }
}
