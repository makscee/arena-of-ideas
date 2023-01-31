use super::*;

/// Load and store shader programs
pub struct ShaderPrograms(HashMap<PathBuf, ugli::Program>);

impl ShaderPrograms {
    pub fn new() -> Self {
        Self(default())
    }

    // full path
    pub fn get_program(&self, path: &PathBuf) -> &ugli::Program {
        &self
            .0
            .get(path)
            .expect(&format!("Shader not loaded {:?}", path))
    }

    pub fn insert_program(&mut self, file: PathBuf, program: ugli::Program) {
        self.0.insert(file, program);
    }
}
