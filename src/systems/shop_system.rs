use geng::prelude::rand::distributions::{Distribution, WeightedIndex};
use geng::ui::*;
use legion::EntityStore;

use super::*;

#[derive(Default)]
pub struct ShopSystem {
    need_reroll: bool,
    need_switch_battle: bool,
}

const UNIT_COST: usize = 3;

impl System for ShopSystem {
    fn post_update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        resources.shop.drag_entity = None;
        resources.shop.drop_entity = None;
    }
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        self.handle_drag(world, resources);
        Self::refresh_cassette(world, resources);
        if self.need_reroll {
            self.need_reroll = false;
            Self::reroll(world, resources);
            resources.shop.money -= 1;
        }
        if self.need_switch_battle {
            resources.camera.focus = match resources.camera.focus {
                Focus::Shop => {
                    BattleSystem::init_battle(world, resources);
                    Focus::Battle
                }
                Focus::Battle => Focus::Shop,
            }
        }
    }

    fn ui<'a>(
        &'a mut self,
        cx: &'a ui::Controller,
        world: &'a legion::World,
        resources: &'a Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        let reroll_btn = Button::new(cx, "Reroll (1G)");
        if reroll_btn.was_clicked() && resources.shop.money > 0 {
            self.need_reroll = true;
        }
        let switch_button = CornerButtonWidget::new(
            cx,
            resources,
            match resources.camera.focus {
                Focus::Shop => resources.options.images.eye_icon.clone(),
                Focus::Battle => resources.options.images.money_icon.clone(),
            },
        );
        self.need_switch_battle = switch_button.was_clicked();
        Box::new(
            (
                (
                    reroll_btn
                        .uniform_padding(16.0)
                        .background_color(Rgba::try_from("#267ec7").unwrap()),
                    Text::new(
                        format!("{}G", resources.shop.money),
                        resources.fonts.get_font(1),
                        60.0,
                        Rgba::BLACK,
                    )
                    .uniform_padding(32.0)
                    .center(),
                )
                    .column()
                    .flex_align(vec2(Some(1.0), None), vec2(1.0, 1.0))
                    .uniform_padding(32.0)
                    .align(vec2(1.0, 1.0)),
                switch_button.place(vec2(1.0, 0.0)),
            )
                .stack(),
        )
    }
}

impl ShopSystem {
    pub fn new() -> Self {
        default()
    }

    pub fn restart(world: &mut legion::World, resources: &mut Resources) {
        UnitSystem::clear_factions(world, resources, &hashset! {Faction::Team});
        resources.status_pool.clear_all_active();
        resources.floors.reset();
        Self::init(world, resources);
    }

    fn refresh_cassette(world: &mut legion::World, resources: &mut Resources) {
        resources.cassette.parallel_node.clear_entities();
        UnitSystem::draw_all_units_to_cassette_node(
            &world,
            &resources.options,
            &resources.status_pool,
            &resources.houses,
            &mut resources.cassette.parallel_node,
            hashset! {Faction::Shop, Faction::Team, Faction::Dark, Faction::Light},
        );
    }

    fn handle_drag(&mut self, world: &mut legion::World, resources: &mut Resources) {
        SlotSystem::set_hovered_slot(world, &Faction::Team, SLOTS_COUNT + 1);
        if let Some(dragged) = resources.shop.drag_entity {
            if let Some(slot) =
                SlotSystem::get_horizontal_hovered_slot(&Faction::Team, resources.input.mouse_pos)
            {
                if SlotSystem::make_gap(world, slot, hashset! {Faction::Team}) {
                    SlotSystem::refresh_slots_filled_uniform(world);
                }
                SlotSystem::set_hovered_slot(world, &Faction::Team, slot);
            }
        }
        if let Some(dropped) = resources.shop.drop_entity {
            resources.shop.drag_entity = None;
            let entry = world.entry(dropped).unwrap();
            let unit = entry.get_component::<UnitComponent>().unwrap();
            match unit.faction {
                Faction::Team => {
                    if entry.get_component::<AreaComponent>().unwrap().position.y > SHOP_POSITION.y
                    {
                        Self::sell(dropped, resources, world);
                    } else if let Some(slot) = SlotSystem::get_horizontal_hovered_slot(
                        &Faction::Team,
                        resources.input.mouse_pos,
                    ) {
                        world
                            .entry_mut(dropped)
                            .unwrap()
                            .get_component_mut::<UnitComponent>()
                            .unwrap()
                            .slot = slot;
                    }
                }
                Faction::Shop => {
                    let slot = SlotSystem::get_horizontal_hovered_slot(
                        &Faction::Team,
                        resources.input.mouse_pos,
                    );
                    if resources.shop.money >= UNIT_COST
                        && slot.is_some()
                        && resources.input.mouse_pos.y < SHOP_POSITION.y + SHOP_CASE_OFFSET.y
                        && Self::team_count(world) < SLOTS_COUNT
                    {
                        let slot = slot.unwrap();
                        Self::buy(dropped, slot, resources, world);
                        ContextSystem::refresh_all(world, resources);
                    }
                }
                _ => {}
            }
            SlotSystem::refresh_slots_filled_uniform(world);
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
        *resources.shop.pool.get_mut(&unit.template_path).unwrap() -= 1;
        Shop::reload_shaders(resources);
        ContextSystem::refresh_entity(entity, world, resources);
        Event::Buy {
            context: ContextSystem::get_context(entity, world),
        }
        .send(resources, world);
    }

    fn sell(entity: legion::Entity, resources: &mut Resources, world: &mut legion::World) {
        resources.shop.money += 1;
        *resources
            .shop
            .pool
            .get_mut(
                &world
                    .entry(entity)
                    .unwrap()
                    .get_component::<UnitComponent>()
                    .unwrap()
                    .template_path,
            )
            .unwrap() += 2;
        Shop::reload_shaders(resources);
        Event::Sell {
            context: ContextSystem::get_context(entity, world),
        }
        .send(resources, world);
        UnitSystem::kill(entity, world, resources);
        SlotSystem::refresh_slots_filled_uniform(world);
    }

    pub fn clear(world: &mut legion::World, resources: &mut Resources) {
        let factions = &hashset! {Faction::Shop};
        UnitSystem::clear_factions(world, resources, factions);
    }

    pub fn init(world: &mut legion::World, resources: &mut Resources) {
        Shop::update_pool(resources);
        Self::reroll(world, resources);
        WorldSystem::set_var(
            world,
            VarName::Floor,
            Var::Int(resources.floors.current_ind() as i32),
        );
        resources.shop.money = (UNIT_COST + resources.floors.current_ind() + 1).min(10);
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
        SlotSystem::refresh_slots_filled_uniform(world);
    }
}
