use super::*;

use std::collections::VecDeque;

pub struct LogicNode {
    pub effects: VecDeque<LogicEffect>,
}
