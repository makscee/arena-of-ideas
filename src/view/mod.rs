use super::*;

mod effect;
mod node;
mod queue;

use effect::*;
use node::*;
use queue::*;

pub struct View {
    pub queue: VisualQueue,
}
