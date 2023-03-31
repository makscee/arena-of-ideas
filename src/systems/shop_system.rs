use geng::ui::*;
use legion::EntityStore;

use super::*;

#[derive(Default)]
pub struct ShopSystem {
    need_switch_battle: bool,
}

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
        let mut tape = Some(Vec::<CassetteNode>::default());
        ActionSystem::run_ticks(world, resources, &mut tape);
        BattleSystem::death_check(&hashset! {Faction::Team}, world, resources, &mut tape);
        ActionSystem::run_ticks(world, resources, &mut tape);
        resources.cassette.add_tape_nodes(tape.unwrap());
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
        let light = Team::pack(&Faction::Team, world, resources);
        let dark = resources.floors.current().clone();
        BattleSystem::init_battle(&light, &dark, world, resources);
    }

    fn switch_to_shop(world: &mut legion::World, resources: &mut Resources) {
        resources.camera.focus = Focus::Shop;
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
            if let Some(entry) = world.entry(dropped) {
                let unit = entry.get_component::<UnitComponent>().unwrap();
                match unit.faction {
                    Faction::Team => {
                        if entry.get_component::<AreaComponent>().unwrap().position.y
                            > SHOP_POSITION.y
                        {
                            resources
                                .shop
                                .pool
                                .push(PackedUnit::pack(dropped, world, resources));
                            ShopSystem::change_g(resources, ShopSystem::sell_price(resources));
                            Self::sell(dropped, resources, world);
                            SlotSystem::refresh_slots_uniforms(world, &resources.options);
                            ContextSystem::refresh_all(world, resources);
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
                        if ShopSystem::get_g(resources) >= ShopSystem::buy_price(resources)
                            && slot.is_some()
                            && resources.input.mouse_pos.y < SHOP_POSITION.y + SHOP_CASE_OFFSET.y
                            && Self::team_count(world) < SLOTS_COUNT
                        {
                            ShopSystem::change_g(resources, -ShopSystem::buy_price(resources));
                            let slot = slot.unwrap();
                            let tape = &mut Some(Vec::<CassetteNode>::default());
                            Self::buy(dropped, slot, resources, world, tape);
                            ContextSystem::refresh_all(world, resources);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn team_count(world: &legion::World) -> usize {
        UnitSystem::collect_faction(world, Faction::Team).len()
    }

    pub fn buy(
        entity: legion::Entity,
        slot: usize,
        resources: &mut Resources,
        world: &mut legion::World,
        nodes: &mut Option<Vec<CassetteNode>>,
    ) {
        let mut entry = world.entry_mut(entity).unwrap();
        let unit = entry.get_component_mut::<UnitComponent>().unwrap();
        unit.faction = Faction::Team;
        unit.slot = slot;

        ContextSystem::refresh_entity(entity, world, resources);
        Event::Buy { owner: entity }.send(world, resources);
        Event::AddToTeam { owner: entity }.send(world, resources);
        ContextSystem::refresh_all(world, resources);
        SlotSystem::move_to_slots_animated(world, resources, nodes);
    }

    pub fn sell(entity: legion::Entity, resources: &mut Resources, world: &mut legion::World) {
        Event::Sell { owner: entity }.send(world, resources);
        UnitSystem::turn_unit_into_corpse(entity, world, resources);
    }

    fn refresh_ui(resources: &mut Resources) {
        let position = SlotSystem::get_position(0, &Faction::Shop);
        let text_color = *resources
            .options
            .colors
            .faction_colors
            .get(&Faction::Shop)
            .unwrap();
        let text = format!("{} g", Self::get_g(resources).to_string());
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

    pub fn floor_money(floor: usize) -> i32 {
        (4 + floor as i32).min(10)
    }

    pub fn get_g(resources: &Resources) -> i32 {
        resources
            .team_states
            .get_vars(&Faction::Team)
            .get_int(&VarName::G)
    }

    pub fn change_g(resources: &mut Resources, delta: i32) {
        resources
            .team_states
            .get_vars_mut(&Faction::Team)
            .change_int(&VarName::G, delta)
    }

    pub fn sell_price(resources: &Resources) -> i32 {
        resources
            .team_states
            .get_vars(&Faction::Team)
            .get_int(&VarName::SellPrice)
    }

    pub fn buy_price(resources: &Resources) -> i32 {
        resources
            .team_states
            .get_vars(&Faction::Team)
            .get_int(&VarName::BuyPrice)
    }

    pub fn reroll_price(resources: &Resources) -> i32 {
        resources
            .team_states
            .get_vars(&Faction::Team)
            .get_int(&VarName::RerollPrice)
    }

    pub fn init_game(world: &mut legion::World, resources: &mut Resources) {
        Shop::load_pool(resources);
        resources.team_states.clear(Faction::Team);
        let vars = resources.team_states.get_vars_mut(&Faction::Team);
        vars.set_int(&VarName::G, 0);
        vars.set_int(&VarName::BuyPrice, 3);
        vars.set_int(&VarName::SellPrice, 1);

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
                        ShopSystem::change_g(resources, -ShopSystem::reroll_price(resources));
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

    pub fn init_floor(world: &mut legion::World, resources: &mut Resources, give_g: bool) {
        let current_floor = resources.floors.current_ind();
        if give_g {
            Self::change_g(resources, Self::floor_money(current_floor));
        }
        Shop::load_floor(resources, current_floor);
        Self::reroll(world, resources);
        WorldSystem::set_var(world, VarName::Floor, Var::Int(current_floor as i32));
        ContextSystem::refresh_all(world, resources);
        Self::refresh_cassette(world, resources);
    }

    pub fn clear_case(world: &mut legion::World, resources: &mut Resources) {
        let case = UnitSystem::collect_faction(world, Faction::Shop);
        let packed_units = case
            .into_iter()
            .map(|entity| PackedUnit::pack(entity, world, resources))
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
