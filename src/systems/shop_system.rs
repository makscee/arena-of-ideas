use geng::prelude::rand::distributions::{Distribution, WeightedIndex};

use super::*;

pub struct ShopSystem {}

impl System for ShopSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if resources.down_keys.contains(&geng::Key::R) {
            Self::refresh(resources);
        }
        resources.cassette.node_template.clear();
        UnitComponent::add_all_units_to_node_template(
            &resources.shop.world,
            &resources.options,
            &resources.statuses,
            &mut resources.cassette.node_template,
        );
    }
}

impl ShopSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn clear(resources: &mut Resources) {
        <(&UnitComponent, &EntityComponent)>::query()
            .iter(&resources.shop.world)
            .filter_map(|(unit, entity)| match unit.faction == Faction::Dark {
                true => Some(entity.entity),
                false => None,
            })
            .collect_vec()
            .iter()
            .for_each(|entity| {
                resources.shop.world.remove(*entity);
            });
    }

    fn get_unit_position(faction: &Faction, slot: usize) -> vec2<f32> {
        vec2(slot as f32, 0.0)
            * match faction {
                Faction::Light => -2.5,
                Faction::Dark => 2.5,
            }
    }

    pub fn refresh(resources: &mut Resources) {
        Self::clear(resources);

        let items = resources.shop.pool.iter().collect_vec();
        let dist2 = WeightedIndex::new(items.iter().map(|item| *item.1)).unwrap();
        for slot in 1..5 {
            let path = items[dist2.sample(&mut thread_rng())].0;
            let template = resources
                .unit_templates
                .get(path)
                .expect(&format!("Failed to find Unit Template: {:?}", path));
            template.create_unit_entity(
                &mut resources.shop.world,
                &mut resources.statuses,
                Faction::Dark,
                slot,
                Self::get_unit_position(&Faction::Dark, slot),
            );
        }
    }
}
