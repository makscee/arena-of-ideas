use super::*;

mod effect;
mod node;
mod queue;

pub use effect::*;
pub use node::*;
pub use queue::*;

pub struct Logic {
    pub queue: LogicQueue,
    pub model: Model,
}
