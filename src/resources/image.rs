use super::*;

#[derive(Debug, Clone, Deserialize)]
pub struct Image {
    pub path: PathBuf, // static path
}

impl Image {
    pub fn get(&self, resources: &Resources) -> Rc<ugli::Texture> {
        resources
            .image_textures
            .get_texture(&static_path().join(&self.path))
            .clone()
    }
}
