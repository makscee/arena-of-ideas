use geng::prelude::itertools::Itertools;

use super::*;

pub const INITIAL_POOL_COUNT_PER_HERO: usize = 5;

#[derive(Default)]
pub struct ShopData {
    pub pool: Vec<PackedUnit>,
    pub floor_extensions: Vec<Vec<PackedUnit>>,
    pub drop_entity: Option<legion::Entity>,
    pub drag_entity: Option<legion::Entity>,
    pub refresh_btn: Option<legion::Entity>,
    pub load_new_hero: bool,
}

impl ShopData {
    pub fn load_pool(resources: &mut Resources) {
        resources.shop_data.pool.clear();
        let mut sorted_by_power = VecDeque::from_iter(resources.hero_pool.all_sorted());
        let heroes_per_extension = (sorted_by_power.len() as f32
            / (resources.ladder.count() as f32 - 3.0))
            .ceil() as usize;
        let mut cur_level = 0;
        resources.shop_data.floor_extensions = vec![default()];
        while let Some(unit) = sorted_by_power.pop_front() {
            if resources
                .shop_data
                .floor_extensions
                .get(cur_level)
                .unwrap()
                .len()
                >= heroes_per_extension
                    + (cur_level == 0) as usize * resources.options.initial_shop_fill
            {
                cur_level += 1;
                resources.shop_data.floor_extensions.push(default());
            }
            resources
                .shop_data
                .floor_extensions
                .get_mut(cur_level)
                .unwrap()
                .push(unit);
        }
        resources
            .shop_data
            .floor_extensions
            .iter()
            .for_each(|x| debug!("{}", x.iter().map(|x| x.to_string()).join(", ")));
    }

    pub fn load_floor(resources: &mut Resources, floor: usize) {
        if let Some(new_units) = resources.shop_data.floor_extensions.get(floor) {
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

    pub fn unpack_pool_unit(
        ind: usize,
        slot: usize,
        resources: &mut Resources,
        world: &mut legion::World,
    ) -> legion::Entity {
        let unit = resources.shop_data.pool.remove(ind);
        unit.unpack(world, resources, slot, Faction::Shop, None)
    }

    pub fn pack_unit_into_pool(
        entity: legion::Entity,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        let unit = PackedUnit::pack(entity, world, resources);
        UnitSystem::delete_unit(entity, world, resources);
        resources.shop_data.pool.push(unit);
    }
}
