pub use super::*;

mod attack;
mod context;
mod effect;
mod game_state;
mod hp;
mod position;
mod shader;
mod trigger;
mod vars;

pub use attack::*;
pub use context::*;
pub use effect::*;
pub use game_state::*;
pub use hp::*;
pub use position::*;
pub use shader::*;
pub use vars::*;

/// Components that can be deserialized from json
#[derive(Deserialize, Debug)]
pub enum Component {
    Hp { max: Hp },
    Attack { value: Hp },
}

impl fmt::Display for Component {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
