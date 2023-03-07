use std::rc::Rc;

use super::*;

/// Load and store image textures
#[derive(Default)]
pub struct ImageTextures(HashMap<PathBuf, Rc<ugli::Texture>>);

impl ImageTextures {
    // full path
    pub fn get_texture(&self, path: &PathBuf) -> &Rc<ugli::Texture> {
        &self
            .0
            .get(path)
            .expect(&format!("Texture not loaded {:?}", path))
    }

    pub fn insert_texture(&mut self, file: PathBuf, texture: ugli::Texture) {
        self.0.insert(file, Rc::new(texture));
    }
}
