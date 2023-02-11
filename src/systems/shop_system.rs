use geng::prelude::rand::distributions::{Distribution, WeightedIndex};
use legion::EntityStore;

use super::*;

pub struct ShopSystem {
    pub buy_candidate: Option<legion::Entity>,
    pub sell_candidate: Option<legion::Entity>,
}

impl System for ShopSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if resources.down_keys.contains(&geng::Key::R) {
            Self::refresh(world, resources);
        }
        resources.cassette.node_template.clear();
        self.handle_drag(world, resources);
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
        Self {
            buy_candidate: default(),
            sell_candidate: default(),
        }
    }

    fn handle_drag(&mut self, world: &mut legion::World, resources: &Resources) {
        if let Some(dragged) = resources.dragged_entity {
            match world
                .entry(dragged)
                .unwrap()
                .get_component::<UnitComponent>()
                .unwrap()
                .faction
            {
                Faction::Team => self.sell_candidate = Some(dragged),
                Faction::Shop => self.buy_candidate = Some(dragged),
                _ => panic!("Wrong faction"),
            }
        } else if let Some(sell_candidate) = self.sell_candidate {
            if world
                .entry(sell_candidate)
                .unwrap()
                .get_component::<Position>()
                .unwrap()
                .0
                .x
                > 0.0
            {
                world.remove(sell_candidate);
            } else {
                if let Some(slot) = SlotSystem::get_mouse_slot(&Faction::Team, resources.mouse_pos)
                {
                    world
                        .entry_mut(sell_candidate)
                        .unwrap()
                        .get_component_mut::<UnitComponent>()
                        .unwrap()
                        .slot = slot;
                }
                SlotSystem::put_unit_into_slot(sell_candidate, world);
            }
            self.sell_candidate = None;
        } else if let Some(buy_candidate) = self.buy_candidate {
            let mut entry = world.entry_mut(buy_candidate).unwrap();
            if let Some(slot) = SlotSystem::get_mouse_slot(&Faction::Team, resources.mouse_pos) {
                entry.get_component_mut::<Position>().unwrap().0 =
                    SlotSystem::get_position(slot, &Faction::Team);
                entry.get_component_mut::<UnitComponent>().unwrap().faction = Faction::Team;
            } else {
                entry.get_component_mut::<Position>().unwrap().0 =
                    SlotSystem::get_unit_position(entry.get_component::<UnitComponent>().unwrap());
            }
            self.buy_candidate = None;
        }
    }

    pub fn clear(world: &mut legion::World, resources: &mut Resources) {
        <(&UnitComponent, &EntityComponent)>::query()
            .iter(world)
            .filter_map(|(unit, entity)| match unit.faction == Faction::Shop {
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
