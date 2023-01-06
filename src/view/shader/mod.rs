use super::*;

mod parameters;
mod program;

pub use parameters::*;
pub use program::*;

#[derive(Deserialize)]
pub struct SystemShaders {
    pub field: ShaderProgram,
}
