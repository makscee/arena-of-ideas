use geng::prelude::itertools::Itertools;

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

    pub async fn load_shaders_list(geng: &Geng) -> Vec<PathBuf> {
        let list =
            <String as geng::LoadAsset>::load(&geng, &static_path().join("shaders/_list.json"))
                .await
                .expect("Failed to load shaders list");
        let list: Vec<String> = serde_json::from_str(&list).expect("Failed to parse shaders list");
        let list = list
            .iter()
            .map(|path| PathBuf::try_from(path).unwrap())
            .collect_vec();
        list
    }
}
