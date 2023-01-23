use super::*;

mod parameters;
mod program;

pub use parameters::*;
pub use program::*;

#[derive(Deserialize, Clone)]
pub struct SystemShaders {
    pub map: HashMap<SystemShader, ShaderProgram>,
}

#[derive(Deserialize, Eq, PartialEq, Hash, Clone)]
pub enum SystemShader {
    Field,
    Unit,
}
