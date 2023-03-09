use super::*;
use geng::prelude::itertools::Itertools;

const HERO_POOL_COUNT: usize = 5;
pub struct Shop {
    pub pool: HashMap<PathBuf, usize>,
    pub level_extensions: Vec<Vec<PathBuf>>,
    pub units: Vec<(String, Shader)>,
    pub money: usize,
}

impl Default for Shop {
    fn default() -> Self {
        Self {
            pool: default(),
            money: default(),
            units: default(),
            level_extensions: default(),
        }
    }
}

impl Shop {
    pub fn load(resources: &mut Resources, world: &mut legion::World) {
        resources.logger.set_enabled(false);
        let mut sorted_heroes = PowerPointsSystem::measure(
            &resources
                .unit_templates
                .heroes
                .keys()
                .cloned()
                .collect_vec(),
            world,
            resources,
        )
        .into_iter()
        .sorted_by_key(|(_, score)| score.clone())
        .rev()
        .collect_vec();
        resources.logger.set_enabled(true);
        dbg!(&sorted_heroes);
        let level_extensions = &mut resources.shop.level_extensions;
        level_extensions.push(default());
        for _ in 0..3 {
            level_extensions[0].push(sorted_heroes.pop().unwrap().0);
        }
        let heroes_per_level = (sorted_heroes.len() as f32 / 10.0).ceil() as usize;
        let mut current_level = 0;
        while let Some((path, _)) = sorted_heroes.pop() {
            level_extensions[current_level].push(path);
            if level_extensions[current_level].len() >= heroes_per_level {
                current_level += 1;
                level_extensions.insert(current_level, default());
            }
        }
        dbg!(level_extensions);
    }

    pub fn update_pool(resources: &mut Resources) {
        let level = resources.rounds.current_ind();
        resources.shop.pool.extend(
            resources.shop.level_extensions[level]
                .iter()
                .map(|path| (path.clone(), HERO_POOL_COUNT)),
        );
        Self::reload_shaders(resources);
    }

    pub fn reset(resources: &mut Resources, world: &mut legion::World) {
        resources.shop = default();
        Self::load(resources, world);
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
