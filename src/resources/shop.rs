use geng::prelude::itertools::Itertools;

use super::*;

const INITIAL_POOL_COUNT_PER_HERO: usize = 5;

#[derive(Default)]
pub struct Shop {
    pub pool: Vec<PackedUnit>,
    pub level_extensions: Vec<Vec<PackedUnit>>,
    pub money: usize,
    pub drop_entity: Option<legion::Entity>,
    pub drag_entity: Option<legion::Entity>,
    pub refresh_btn: Option<legion::Entity>,
}

impl Shop {
    pub fn load_pool(world: &mut legion::World, resources: &mut Resources) {
        let measures = PowerPointsSystem::measure(resources.hero_pool.all(), world, resources);
        dbg!(measures
            .iter()
            .map(|(unit, score)| (&unit.name, score))
            .sorted_by_key(|x| x.1)
            .collect_vec());
        let mut sorted_by_power = VecDeque::from_iter(
            measures
                .into_iter()
                .sorted_by_key(|(_, score)| *score)
                .map(|(unit, _)| unit),
        );
        let heroes_per_extension = (sorted_by_power.len() as f32 / 10.0).ceil() as usize;
        let mut cur_level = 0;
        resources.shop.level_extensions = vec![default()];
        while let Some(unit) = sorted_by_power.pop_front() {
            if resources
                .shop
                .level_extensions
                .get(cur_level)
                .unwrap()
                .len()
                >= heroes_per_extension
                    + (cur_level == 0) as usize * resources.options.initial_shop_fill
            {
                cur_level += 1;
                resources.shop.level_extensions.push(default());
            }
            resources
                .shop
                .level_extensions
                .get_mut(cur_level)
                .unwrap()
                .push(unit);
        }
    }

    pub fn load_level(resources: &mut Resources, level: usize) {
        if let Some(new_units) = resources.shop.level_extensions.get(level) {
            resources.shop.pool.extend(
                new_units
                    .iter()
                    .map(|unit| {
                        (0..INITIAL_POOL_COUNT_PER_HERO)
                            .map(|_| unit.clone())
                            .collect_vec()
                    })
                    .flatten(),
            )
        }
    }
}
