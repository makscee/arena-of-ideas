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

    fn texture_loader(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::texture_loader));
        let geng = resources.geng.as_ref().unwrap();
        let texture =
            futures::executor::block_on(<ugli::Texture as geng::LoadAsset>::load(geng, &path));
        match texture {
            Ok(texture) => {
                debug!("Load texture {:?}", path);
                resources
                    .image_textures
                    .insert_texture(path.clone(), texture)
            }
            Err(error) => {
                error!("Failed to load texture {:?} {}", path, error);
            }
        }
    }
}

impl FileWatcherLoader for ImageTextures {
    fn loader(resources: &mut Resources, path: &PathBuf, watcher: &mut FileWatcherSystem) {
        watcher.watch_file(path, Box::new(Self::loader));
        let paths: Vec<PathBuf> = futures::executor::block_on(load_json(path)).unwrap();
        paths.into_iter().for_each(|path| {
            Self::texture_loader(resources, &static_path().join(path), watcher);
        })
    }
}
