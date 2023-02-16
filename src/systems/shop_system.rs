use geng::prelude::rand::distributions::{Distribution, WeightedIndex};
use legion::EntityStore;

use super::*;

const GRAB_ANIMATION_DURATION: Time = 0.15;

pub struct ShopSystem {
    buy_candidate: Option<legion::Entity>,
    sell_candidate: Option<legion::Entity>,
    hovered_team: Option<legion::Entity>,
}

impl System for ShopSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if resources.down_keys.contains(&geng::Key::R) {
            Self::refresh(world, resources);
        }
        self.handle_hover(world, resources);
        self.handle_drag(world, resources);
        Self::refresh_cassette(world, resources);
    }
}

impl ShopSystem {
    pub fn new() -> Self {
        Self {
            buy_candidate: default(),
            sell_candidate: default(),
            hovered_team: default(),
        }
    }

    fn refresh_cassette(world: &mut legion::World, resources: &mut Resources) {
        resources.cassette.parallel_node.clear_entities();
        UnitComponent::draw_all_units_to_cassette_node(
            &world,
            &resources.options,
            &resources.status_pool,
            &mut resources.cassette.parallel_node,
            hashset! {Faction::Shop, Faction::Team},
        );
    }

    fn handle_hover(&mut self, world: &legion::World, resources: &mut Resources) {
        let hovered = resources.hovered_entity.or(resources.dragged_entity);
        if self.hovered_team == hovered {
            return;
        }
        if let Some(entry) = hovered.and_then(|entity| world.entry_ref(entity).ok()) {
            let hovered = hovered.unwrap();
            if entry.get_component::<UnitComponent>().unwrap().faction == Faction::Team {
                resources.cassette.close_node();
                resources.cassette.head = resources.cassette.last_start();
                resources.cassette.add_effect(VisualEffect {
                    duration: GRAB_ANIMATION_DURATION,
                    r#type: VisualEffectType::EntityShaderAnimation {
                        entity: hovered,
                        from: hashmap! {"u_card" => ShaderUniform::Float(0.0)}.into(),
                        to: hashmap! {"u_card" => ShaderUniform::Float(1.0)}.into(),
                        easing: EasingType::Linear,
                    },
                    order: -1,
                });
                resources.cassette.parallel_node.add_effect_by_key(
                    "hover_card_fix",
                    VisualEffect {
                        duration: 0.0,
                        r#type: VisualEffectType::EntityShaderConst {
                            entity: hovered,
                            uniforms: hashmap! {"u_card" => ShaderUniform::Float(1.0)}.into(),
                        },
                        order: -2,
                    },
                );
                resources.cassette.close_node();
                self.hovered_team = Some(hovered);
            }
        }
        if hovered.is_none() && self.hovered_team.is_some() {
            resources.cassette.parallel_node.clear_key("hover_card_fix");
            resources.cassette.close_node();
            let hovered = self.hovered_team.unwrap();
            resources.cassette.add_effect(VisualEffect {
                duration: GRAB_ANIMATION_DURATION,
                r#type: VisualEffectType::EntityShaderAnimation {
                    entity: hovered,
                    from: hashmap! {"u_card" => ShaderUniform::Float(1.0)}.into(),
                    to: hashmap! {"u_card" => ShaderUniform::Float(0.0)}.into(),
                    easing: EasingType::Linear,
                },
                order: -1,
            });
            resources.cassette.close_node();
            self.hovered_team = None;
        }
    }

    fn handle_drag(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if let Some(dragged) = resources.dragged_entity {
            if let Some(slot) =
                SlotSystem::get_horizontal_hovered_slot(&Faction::Team, resources.mouse_pos)
            {
                resources.cassette.close_node();
                resources.cassette.head = resources.cassette.last_start();
                SlotSystem::make_gap(world, resources, slot, hashset! {Faction::Team});
                resources.cassette.close_node();
            }
            match world
                .entry(dragged)
                .unwrap()
                .get_component::<UnitComponent>()
                .unwrap()
                .faction
            {
                Faction::Team => {
                    if self.sell_candidate.is_none() {
                        self.sell_candidate = Some(dragged);
                    }
                }
                Faction::Shop => {
                    if self.buy_candidate.is_none() {
                        self.buy_candidate = Some(dragged);
                    }
                }
                _ => panic!("Wrong faction"),
            }
        } else if let Some(sell_candidate) = self.sell_candidate {
            if world
                .entry(sell_candidate)
                .unwrap()
                .get_component::<PositionComponent>()
                .unwrap()
                .0
                .x
                > 0.0
            {
                world.remove(sell_candidate);
            } else {
                if let Some(slot) =
                    SlotSystem::get_horizontal_hovered_slot(&Faction::Team, resources.mouse_pos)
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
            SlotSystem::refresh_slot_shaders(
                world,
                resources,
                hashset! {Faction::Shop,Faction::Team},
            );
        } else if let Some(buy_candidate) = self.buy_candidate {
            let mut entry = world.entry_mut(buy_candidate).unwrap();
            if let Some(slot) =
                SlotSystem::get_horizontal_hovered_slot(&Faction::Team, resources.mouse_pos)
            {
                let unit = entry.get_component_mut::<UnitComponent>().unwrap();
                unit.faction = Faction::Team;
                unit.slot = slot;
                SlotSystem::put_unit_into_slot(buy_candidate, world);
                Self::refresh_cassette(world, resources);
                self.hovered_team = Some(buy_candidate);
                resources.cassette.parallel_node.add_effect_by_key(
                    "hover_card_fix",
                    VisualEffect {
                        duration: 0.0,
                        r#type: VisualEffectType::EntityShaderConst {
                            entity: buy_candidate,
                            uniforms: hashmap! {"u_card" => ShaderUniform::Float(1.0)}.into(),
                        },
                        order: -2,
                    },
                );
            } else {
                let position =
                    SlotSystem::get_unit_position(entry.get_component::<UnitComponent>().unwrap());
                entry.get_component_mut::<PositionComponent>().unwrap().0 = position;
            }
            self.buy_candidate = None;
            SlotSystem::refresh_slot_shaders(
                world,
                resources,
                hashset! {Faction::Shop,Faction::Team},
            );
        }
    }

    pub fn clear(world: &mut legion::World, _resources: &mut Resources) {
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

        let items = resources
            .shop
            .pool
            .iter()
            .map(|(path, size)| (path.clone(), *size))
            .collect_vec();
        let dist2 = WeightedIndex::new(items.iter().map(|item| item.1)).unwrap();
        for slot in 1..5 {
            let path = &items[dist2.sample(&mut thread_rng())].0;
            UnitTemplatesPool::create_unit_entity(
                path,
                resources,
                world,
                Faction::Shop,
                slot,
                SlotSystem::get_position(slot, &Faction::Shop),
            );
        }
        SlotSystem::refresh_slot_shaders(world, resources, hashset! {Faction::Shop,Faction::Team});
    }
}
