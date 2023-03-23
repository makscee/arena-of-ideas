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
        SlotSystem::refresh_slots_uniforms(world, &resources.options);
        if self.need_switch_battle {
            match resources.camera.focus {
                Focus::Shop => {
                    Self::switch_to_battle(world, resources);
                }
                Focus::Battle => {
                    Self::switch_to_shop(world, resources);
                }
            }
            self.need_switch_battle = false;
        }
        Self::refresh_ui(resources);
        Self::refresh_cassette(world, resources);
        let mut nodes = ActionSystem::run_ticks(world, resources);
        if !nodes.is_empty() {
            BattleSystem::death_check(world, resources);
            nodes.extend(ActionSystem::run_ticks(world, resources));
            resources.cassette.add_tape_nodes(nodes);
        }
    }

    fn ui<'a>(
        &'a mut self,
        cx: &'a ui::Controller,
        _: &'a legion::World,
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
        self.need_switch_battle = switch_button.was_clicked() || self.need_switch_battle;
        Box::new((switch_button.place(vec2(1.0, 0.0)),).stack())
    }
}

impl ShopSystem {
    pub fn new() -> Self {
        default()
    }

    pub fn switch_to_battle(world: &mut legion::World, resources: &mut Resources) {
        resources.camera.focus = Focus::Battle;
        TeamPool::refresh_team(&Faction::Team, world, resources);
        BattleSystem::save_player_team(resources);
        BattleSystem::save_floor(resources);
        BattleSystem::init_battle(world, resources);
    }

    fn switch_to_shop(world: &mut legion::World, resources: &mut Resources) {
        resources.camera.focus = Focus::Shop;
    }

    pub fn restart(world: &mut legion::World, resources: &mut Resources) {
        UnitSystem::clear_factions(world, resources, &hashset! {Faction::Team});
        resources.status_pool.clear_all_active();
        resources.floors.reset();
        Self::init(world, resources);
    }

    fn handle_drag(&mut self, world: &mut legion::World, resources: &mut Resources) {
        SlotSystem::set_hovered_slot(world, &Faction::Team, SLOTS_COUNT + 1);
        if let Some(dragged) = resources.shop.drag_entity {
            if let Some(slot) =
                SlotSystem::get_horizontal_hovered_slot(&Faction::Team, resources.input.mouse_pos)
            {
                if SlotSystem::make_gap(world, resources, slot, &hashset! {Faction::Team}) {
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
                        SlotSystem::fill_gaps(world, resources, &hashset! {Faction::Team});
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
        let node = &mut default();
        VfxSystem::translate_animated(
            entity,
            SlotSystem::get_unit_position(unit),
            node,
            world,
            EasingType::QuartInOut,
            0.5,
        );
        resources.cassette.add_tape_nodes(vec![node.to_owned()]);
        ContextSystem::refresh_entity(entity, world, resources);
        Event::Buy { owner: entity }.send(world, resources);
        Event::AddToTeam { owner: entity }.send(world, resources);
        TeamPool::refresh_team(&Faction::Team, world, resources);
        ContextSystem::refresh_all(world, resources);
    }

    fn sell(entity: legion::Entity, resources: &mut Resources, world: &mut legion::World) {
        resources.shop.money += 1;
        Event::Sell { owner: entity }.send(world, resources);
        resources
            .shop
            .pool
            .push(PackedUnit::pack(entity, world, resources));
        UnitSystem::turn_unit_into_corpse(entity, world, resources);
        SlotSystem::refresh_slots_uniforms(world, &resources.options);
        TeamPool::refresh_team(&Faction::Team, world, resources);
        ContextSystem::refresh_all(world, resources);
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
                .set_uniform("u_color", ShaderUniform::Color(text_color))
                .set_uniform("u_text", ShaderUniform::String((1, text))),
        )
    }

    fn refresh_cassette(world: &legion::World, resources: &mut Resources) {
        let mut node = CassetteNode::default();
        UnitSystem::draw_all_units_to_cassette_node(
            &hashset! {Faction::Shop, Faction::Team, Faction::Light, Faction::Dark},
            &mut node,
            world,
            resources,
        );
        resources.cassette.render_node = node;
    }

    pub fn init(world: &mut legion::World, resources: &mut Resources) {
        let current_floor = resources.floors.current_ind();
        Shop::load_level(resources, current_floor);
        Self::reroll(world, resources);
        WorldSystem::set_var(world, VarName::Floor, Var::Int(current_floor as i32));
        resources.shop.money = (UNIT_COST + current_floor + 1).min(10);
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
        ContextSystem::refresh_all(world, resources);
        Self::refresh_cassette(world, resources);
    }

    pub fn clear_case(world: &mut legion::World, resources: &mut Resources) {
        let case = UnitSystem::collect_faction(world, Faction::Shop);
        let packed_units = case
            .keys()
            .map(|entity| PackedUnit::pack(*entity, world, resources))
            .collect_vec();
        UnitSystem::clear_faction(world, resources, Faction::Shop);
        resources.shop.pool.extend(packed_units.into_iter());
    }

    pub fn fill_case(world: &mut legion::World, resources: &mut Resources) {
        for slot in 0..SLOTS_COUNT {
            if resources.shop.pool.is_empty() {
                return;
            }
            let slot = slot + 1;
            let mut rng = rand::thread_rng();
            let ind: usize = rng.gen_range(0..resources.shop.pool.len());
            let position = SlotSystem::get_position(slot, &Faction::Shop);
            resources.shop.pool.remove(ind).unpack(
                world,
                resources,
                slot,
                Faction::Shop,
                Some(position),
            );
        }
    }

    pub fn reroll(world: &mut legion::World, resources: &mut Resources) {
        Self::clear_case(world, resources);
        Self::fill_case(world, resources);
        SlotSystem::refresh_slots_uniforms(world, &resources.options);
    }
}
