use geng::prelude::itertools::Itertools;

use super::*;

pub const INITIAL_POOL_COUNT_PER_HERO: usize = 5;

#[derive(Default)]
pub struct ShopData {
    pub pool: Vec<PackedUnit>,
    pub offered: Vec<PackedUnit>,
    pub level_extensions: Vec<Vec<PackedUnit>>,
    pub load_new_hero: bool,
    pub status_apply: Option<(String, i32, BuffTarget)>,
    pub current_team_size: usize,
    pub loaded: bool,
}

impl ShopData {
    pub fn load_pool(resources: &mut Resources) {
        resources.shop_data.pool.clear();
        let mut sorted_by_rating = VecDeque::from_iter(HeroPool::all_sorted(&resources));
        let heroes_per_extension = (sorted_by_rating.len() as f32 / (6.0)).ceil() as usize;
        let mut cur_level = 0;
        resources.shop_data.level_extensions = vec![default()];
        while let Some(unit) = sorted_by_rating.pop_front() {
            if resources
                .shop_data
                .level_extensions
                .get(cur_level)
                .unwrap()
                .len()
                >= heroes_per_extension
                    + (cur_level == 0) as usize * resources.options.initial_shop_fill
            {
                cur_level += 1;
                resources.shop_data.level_extensions.push(default());
            }
            resources
                .shop_data
                .level_extensions
                .get_mut(cur_level)
                .unwrap()
                .push(unit);
        }
        resources
            .shop_data
            .level_extensions
            .iter()
            .for_each(|x| debug!("{}", x.iter().map(|x| x.to_string()).join("\n")));
    }

    pub fn load_pool_full(resources: &mut Resources) {
        resources.shop_data.pool = HeroPool::all(&resources);
    }

    pub fn load_level(resources: &mut Resources, level: usize) {
        if let Some(new_units) = resources.shop_data.level_extensions.get(level) {
            resources.shop_data.pool.extend(
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

    pub fn pool_len(resources: &Resources) -> usize {
        resources.shop_data.pool.len()
    }
}
