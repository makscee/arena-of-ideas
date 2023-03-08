use super::*;
use geng::prelude::itertools::Itertools;

pub struct Shop {
    pub pool: HashMap<PathBuf, usize>,
    pub units: Vec<(String, Shader)>,
    pub money: usize,
}

impl Default for Shop {
    fn default() -> Self {
        Self {
            pool: default(),
            money: default(),
            units: default(),
        }
    }
}

impl Shop {
    pub fn load(resources: &mut Resources) {
        resources.shop.pool = HashMap::from_iter(
            resources
                .unit_templates
                .heroes
                .keys()
                .map(|path| (path.clone(), 3)),
        );
        Self::reload_shaders(resources);
    }

    pub fn reload_shaders(resources: &mut Resources) {
        resources.shop.units = resources
            .shop
            .pool
            .iter()
            .map(|(path, size)| {
                resources
                    .unit_templates
                    .heroes
                    .get(path)
                    .unwrap()
                    .0
                    .iter()
                    .filter_map(|component| SerializedComponent::unpack_shader(component))
                    .map(|shader| (size.to_string(), shader))
                    .collect_vec()
            })
            .flatten()
            .collect_vec();
        resources.fonts.load_textures(
            resources
                .shop
                .units
                .iter()
                .map(|(size, _)| (1usize, size))
                .collect_vec(),
        );
    }
}
