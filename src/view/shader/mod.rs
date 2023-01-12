use super::*;

mod parameters;
mod program;

pub use parameters::*;
pub use program::*;

#[derive(Deserialize, Clone)]
pub struct SystemShaders {
    pub field: ShaderProgram,
    pub unit: ShaderProgram,
}
