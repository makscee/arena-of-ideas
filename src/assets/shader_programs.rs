use geng::prelude::itertools::Itertools;

use super::*;

/// Load and store shader programs
pub struct ShaderPrograms(pub HashMap<PathBuf, ugli::Program>);

impl ShaderPrograms {
    pub fn new(geng: &Geng) -> Self {
        let list = futures::executor::block_on(Self::get_shaders_list(&geng));
        let shaders = futures::executor::block_on(Self::load_shaders(&geng, list));
        Self(shaders)
    }

    pub fn get_program(&self, path: &PathBuf) -> &ugli::Program {
        &self
            .0
            .get(path)
            .expect(&format!("Shader not loaded {:?}", path))
    }
    fn get_path() -> PathBuf {
        static_path().join("shaders")
    }

    async fn get_shaders_list(geng: &Geng) -> Vec<PathBuf> {
        let list = <String as geng::LoadAsset>::load(&geng, &Self::get_path().join("_list.json"))
            .await
            .expect("Failed to load shaders list");
        let list: Vec<String> = serde_json::from_str(&list).expect("Failed to parse shaders list");
        let list = list
            .iter()
            .map(|path| PathBuf::try_from(path).unwrap())
            .collect_vec();
        list
    }

    async fn load_shaders(geng: &Geng, list: Vec<PathBuf>) -> HashMap<PathBuf, ugli::Program> {
        let mut result: HashMap<PathBuf, ugli::Program> = HashMap::default();
        for path in list.iter() {
            result.insert(
                path.clone(),
                <ugli::Program as LoadAsset>::load(&geng, &Self::get_path().join(path))
                    .await
                    .expect(&format!("Failed to load shader {:?}", path)),
            );
        }
        result
    }
}
