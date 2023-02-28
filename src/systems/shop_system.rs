use geng::prelude::rand::distributions::{Distribution, WeightedIndex};
use geng::ui::*;
use legion::EntityStore;

use super::*;

pub struct ShopSystem {
    buy_candidate: Option<legion::Entity>,
    sell_candidate: Option<legion::Entity>,
    hovered_team: Option<legion::Entity>,
}

const UNIT_COST: usize = 3;

impl System for ShopSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if resources.down_keys.contains(&geng::Key::R) {
            Self::reroll(world, resources);
        }
        if resources.down_keys.contains(&geng::Key::C) {
            resources.shop.money += 10;
        }
        self.handle_drag(world, resources);
        Self::refresh_cassette(world, resources);
    }

    fn ui<'a>(
        &'a mut self,
        cx: &'a ui::Controller,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        let reroll_btn = Button::new(cx, "Reroll (1G)");
        if reroll_btn.was_clicked() && resources.shop.money > 0 {
            resources.shop.money -= 1;
            Self::reroll(world, resources);
        }
        Box::new(
            (
                Text::new(
                    format!("Round #{}", resources.rounds.next_round + 1),
                    resources.fonts.get_font(0),
                    70.0,
                    Rgba::WHITE,
                ),
                Text::new(
                    format!("Money: {}G", resources.shop.money),
                    resources.fonts.get_font(1),
                    70.0,
                    Rgba::WHITE,
                ),
                reroll_btn
                    .uniform_padding(16.0)
                    .background_color(Rgba::try_from("#267ec7").unwrap()),
                Text::new(
                    format!("Buy: {}G", UNIT_COST),
                    resources.fonts.get_font(1),
                    40.0,
                    Rgba::WHITE,
                ),
            )
                .column()
                .flex_align(vec2(Some(1.0), None), vec2(1.0, 1.0))
                .uniform_padding(32.0)
                .align(vec2(1.0, 1.0)),
        )
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

    pub fn restart(world: &mut legion::World, resources: &mut Resources) {
        resources.rounds.next_round = 0;
        resources.shop.money = 4;
        WorldSystem::clear_factions(world, &hashset! {Faction::Team});
    }

    fn refresh_cassette(world: &mut legion::World, resources: &mut Resources) {
        resources.cassette.parallel_node.clear_entities();
        UnitSystem::draw_all_units_to_cassette_node(
            &world,
            &resources.options,
            &resources.status_pool,
            &mut resources.cassette.parallel_node,
            hashset! {Faction::Shop, Faction::Team},
        );
    }

    fn handle_drag(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if let Some(dragged) = resources.dragged_entity {
            if let Some(slot) =
                SlotSystem::get_horizontal_hovered_slot(&Faction::Team, resources.mouse_pos)
            {
                resources.cassette.close_node();
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
                Self::sell(sell_candidate, resources, world);
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
                hashset! {Faction::Shop, Faction::Team},
            );
        } else if let Some(buy_candidate) = self.buy_candidate {
            let slot = SlotSystem::get_horizontal_hovered_slot(&Faction::Team, resources.mouse_pos);
            if resources.shop.money >= UNIT_COST
                && slot.is_some()
                && Self::team_count(world) < SLOTS_COUNT
            {
                let slot = slot.unwrap();
                Self::buy(buy_candidate, slot, resources, world);
                SlotSystem::put_unit_into_slot(buy_candidate, world);
                Self::refresh_cassette(world, resources);
                self.hovered_team = Some(buy_candidate);
            } else {
                let mut entry = world.entry_mut(buy_candidate).unwrap();
                let position =
                    SlotSystem::get_unit_position(entry.get_component::<UnitComponent>().unwrap());
                entry.get_component_mut::<PositionComponent>().unwrap().0 = position;
            }
            self.buy_candidate = None;
            SlotSystem::fill_gaps(world, resources, hashset! {Faction::Team});
            SlotSystem::refresh_slot_shaders(
                world,
                resources,
                hashset! {Faction::Shop,Faction::Team},
            );
        }
    }

    fn team_count(world: &legion::World) -> usize {
        <&UnitComponent>::query()
            .iter(world)
            .filter(|unit| unit.faction == Faction::Team)
            .count()
    }

    fn buy(
        entity: legion::Entity,
        slot: usize,
        resources: &mut Resources,
        world: &mut legion::World,
    ) {
        resources.shop.money -= UNIT_COST;
        let mut entry = world.entry_mut(entity).unwrap();
        let unit = entry.get_component_mut::<UnitComponent>().unwrap();
        unit.faction = Faction::Team;
        unit.slot = slot;
        Event::Buy {
            context: ContextSystem::get_context(entity, world),
        }
        .send(resources, world);
    }

    fn sell(entity: legion::Entity, resources: &mut Resources, world: &mut legion::World) {
        resources.shop.money += 1;
        Event::Sell {
            context: ContextSystem::get_context(entity, world),
        }
        .send(resources, world);
        WorldSystem::kill(entity, world, resources);
    }

    pub fn clear(world: &mut legion::World, resources: &mut Resources) {
        let factions = &hashset! {Faction::Shop};
        resources.status_pool.clear_factions(factions, world);
        WorldSystem::clear_factions(world, factions);
    }

    pub fn init(world: &mut legion::World, resources: &mut Resources) {
        Self::reroll(world, resources);
        resources.shop.money = (UNIT_COST + 1 + resources.rounds.next_round).min(10);
    }

    pub fn reroll(world: &mut legion::World, resources: &mut Resources) {
        Self::clear(world, resources);

        let items = resources
            .shop
            .pool
            .iter()
            .map(|(path, size)| (path.clone(), *size))
            .collect_vec();
        let dist2 = WeightedIndex::new(items.iter().map(|item| item.1)).unwrap();
        for slot in 1..=SLOTS_COUNT {
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
