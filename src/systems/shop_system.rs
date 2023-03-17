use geng::prelude::rand::distributions::{Distribution, WeightedIndex};
use geng::ui::*;
use legion::EntityStore;

use super::*;

#[derive(Default)]
pub struct ShopSystem {
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
        SlotSystem::refresh_slots_uniforms(world, &resources.options);
        if self.need_switch_battle {
            resources.camera.focus = match resources.camera.focus {
                Focus::Shop => {
                    BattleSystem::init_battle(world, resources);
                    Focus::Battle
                }
                Focus::Battle => Focus::Shop,
            }
        }
        Self::refresh_ui(resources);
    }

    fn ui<'a>(
        &'a mut self,
        cx: &'a ui::Controller,
        world: &'a legion::World,
        resources: &'a Resources,
    ) -> Box<dyn ui::Widget + 'a> {
        let switch_button = CornerButtonWidget::new(
            cx,
            resources,
            match resources.camera.focus {
                Focus::Shop => resources.options.images.eye_icon.clone(),
                Focus::Battle => resources.options.images.money_icon.clone(),
            },
        );
        self.need_switch_battle = switch_button.was_clicked();
        Box::new((switch_button.place(vec2(1.0, 0.0)),).stack())
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
                if SlotSystem::make_gap(world, resources, slot, hashset! {Faction::Team}) {
                    SlotSystem::refresh_slots_uniforms(world, &resources.options);
                }
                SlotSystem::set_hovered_slot(world, &Faction::Team, slot);
            }
        }
        if let Some(dropped) = resources.shop.drop_entity {
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
                    } else {
                        SlotSystem::fill_gaps(world, resources, hashset! {Faction::Team});
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
            SlotSystem::refresh_slots_uniforms(world, &resources.options);
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
        SlotSystem::refresh_slots_uniforms(world, &resources.options);
    }

    pub fn clear(world: &mut legion::World, resources: &mut Resources) {
        let factions = &hashset! {Faction::Shop};
        UnitSystem::clear_factions(world, resources, factions);
    }

    fn refresh_ui(resources: &mut Resources) {
        let position = SlotSystem::get_position(0, &Faction::Shop);
        let text_color = *resources
            .options
            .colors
            .faction_colors
            .get(&Faction::Shop)
            .unwrap();
        let text = format!("{} g", resources.shop.money.to_string());
        resources.frame_shaders.push(
            resources
                .options
                .shaders
                .money_indicator
                .clone()
                .set_uniform("u_position", ShaderUniform::Vec2(position))
                .set_uniform("u_text_color", ShaderUniform::Color(text_color))
                .set_uniform("u_text", ShaderUniform::String((1, text))),
        )
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
        if resources.shop.refresh_btn.is_none() {
            let world_entity = WorldSystem::get_context(world).owner;
            fn refresh(
                entity: legion::Entity,
                resources: &mut Resources,
                world: &mut legion::World,
                event: InputEvent,
            ) {
                match event {
                    InputEvent::Click => {
                        ShopSystem::reroll(world, resources);
                        resources.shop.money -= 1;
                    }
                    InputEvent::HoverStart => ButtonSystem::change_icon_color(
                        entity,
                        world,
                        resources.options.colors.btn_hovered,
                    ),
                    InputEvent::HoverStop => ButtonSystem::change_icon_color(
                        entity,
                        world,
                        resources.options.colors.btn_normal,
                    ),
                    _ => {}
                }
            }

            ButtonSystem::create_button(
                world,
                world_entity,
                resources,
                resources.options.images.refresh_icon.clone(),
                resources.options.colors.btn_normal,
                refresh,
                SlotSystem::get_position(0, &Faction::Shop) + vec2(0.0, -2.0),
                &hashmap! {
                    "u_scale" => ShaderUniform::Float(1.1),
                }
                .into(),
            );
        }
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
        SlotSystem::refresh_slots_uniforms(world, &resources.options);
    }
}
