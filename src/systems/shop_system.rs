use geng::prelude::rand::distributions::{Distribution, WeightedIndex};

use super::*;

pub struct ShopSystem {}

impl System for ShopSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if resources.down_keys.contains(&geng::Key::R) {
            Self::refresh(world, resources);
        }
        resources.cassette.node_template.clear();
        UnitComponent::add_all_units_to_node_template(
            &world,
            &resources.options,
            &resources.statuses,
            &mut resources.cassette.node_template,
            hashset! {Faction::Shop, Faction::Team},
        );
    }
}

impl ShopSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn clear(world: &mut legion::World, resources: &mut Resources) {
        <(&UnitComponent, &EntityComponent)>::query()
            .iter(world)
            .filter_map(|(unit, entity)| match unit.faction == Faction::Dark {
                true => Some(entity.entity),
                false => None,
            })
            .collect_vec()
            .iter()
            .for_each(|entity| {
                world.remove(*entity);
            });
    }

    pub fn refresh(world: &mut legion::World, resources: &mut Resources) {
        Self::clear(world, resources);

        let items = resources.shop.pool.iter().collect_vec();
        let dist2 = WeightedIndex::new(items.iter().map(|item| *item.1)).unwrap();
        for slot in 1..5 {
            let path = items[dist2.sample(&mut thread_rng())].0;
            let template = resources
                .unit_templates
                .get(path)
                .expect(&format!("Failed to find Unit Template: {:?}", path));
            template.create_unit_entity(
                world,
                &mut resources.statuses,
                Faction::Shop,
                slot,
                SlotSystem::get_position(slot, &Faction::Shop),
            );
        }
    }
}
