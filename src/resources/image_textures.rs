use super::*;

/// Load and store image textures
#[derive(Default)]
pub struct ImageTextures(HashMap<PathBuf, ugli::Texture>);

impl ImageTextures {
    // full path
    pub fn get_texture(&self, image: &Image) -> Option<&ugli::Texture> {
        self.0.get(&static_path().join(&image.path))
    }

    pub fn insert_texture(&mut self, file: PathBuf, texture: ugli::Texture) {
        self.0.insert(file, texture);
    }
}
