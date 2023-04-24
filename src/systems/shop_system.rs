use geng::ui::*;
use legion::EntityStore;

use super::*;

#[derive(Default)]
pub struct ShopSystem {
    need_switch_battle: bool,
}

impl System for ShopSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
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
        Self::refresh_tape(world, resources);
        if !resources.action_queue.is_empty() {
            let mut cluster = Some(NodeCluster::default());
            ActionSystem::run_ticks(world, resources, &mut cluster);
            BattleSystem::death_check(world, resources, &mut cluster);
            ActionSystem::run_ticks(world, resources, &mut cluster);

            resources
                .tape_player
                .tape
                .push_to_queue(cluster.unwrap(), resources.tape_player.head);
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
    fn draw(&self, world: &legion::World, resources: &mut Resources, _: &mut ugli::Framebuffer) {
        let position = SlotSystem::get_position(0, &Faction::Shop, resources);
        let text_color = *resources
            .options
            .colors
            .factions
            .get(&Faction::Shop)
            .unwrap();
        let text = format!("{} g", Self::get_g(world).to_string());
        let money_indicator = &resources.options.shaders.money_indicator;
        resources.frame_shaders.push(
            money_indicator
                .clone()
                .set_uniform("u_position", ShaderUniform::Vec2(position))
                .set_uniform("u_color", ShaderUniform::Color(text_color))
                .set_uniform("u_text", ShaderUniform::String((0, text))),
        );
        let text = format!("{} g", Self::reroll_price(world, resources).to_string());
        let money_indicator = &resources.options.shaders.money_indicator;
        resources.frame_shaders.push(
            money_indicator
                .clone()
                .set_uniform("u_size", ShaderUniform::Float(0.5))
                .set_uniform(
                    "u_position",
                    ShaderUniform::Vec2(Self::reroll_btn_position(resources) + vec2(1.5, 0.0)),
                )
                .set_uniform("u_color", ShaderUniform::Color(text_color))
                .set_uniform("u_text", ShaderUniform::String((0, text))),
        );
    }
}

impl ShopSystem {
    pub fn new() -> Self {
        default()
    }

    pub fn switch_to_battle(world: &mut legion::World, resources: &mut Resources) {
        resources.camera.focus = Focus::Battle;
        let light = PackedTeam::pack(&Faction::Team, world, resources);
        let dark = Ladder::load_team(resources);
        BattleSystem::init_battle(&light, &dark, world, resources);
    }

    fn switch_to_shop(_: &mut legion::World, resources: &mut Resources) {
        resources.camera.focus = Focus::Shop;
    }

    fn team_full(world: &legion::World, resources: &Resources) -> bool {
        let faction = Faction::Team;
        UnitSystem::collect_faction(world, faction).len()
            >= TeamSystem::get_state(&faction, world)
                .vars
                .get_int(&VarName::Slots) as usize
    }

    pub fn try_buy(
        entity: legion::Entity,
        slot: usize,
        resources: &mut Resources,
        world: &mut legion::World,
    ) {
        if !Self::team_full(world, resources) && Self::get_g(world) >= Self::buy_price(world) {
            Self::do_buy(entity, slot, resources, world);
            Self::change_g(-Self::buy_price(world), world);
        }
    }

    pub fn do_buy(
        entity: legion::Entity,
        slot: usize,
        resources: &mut Resources,
        world: &mut legion::World,
    ) {
        let team = TeamSystem::entity(&Faction::Team, world);
        let state = ContextState::get_mut(entity, world);
        state.parent = team;
        state.vars.set_int(&VarName::Slot, slot as i32);

        Event::Buy { owner: entity }.send(world, resources);
        Event::AddToTeam { owner: entity }.send(world, resources);
    }

    pub fn try_sell(entity: legion::Entity, resources: &mut Resources, world: &mut legion::World) {
        Self::change_g(Self::sell_price(world), world);
        resources
            .shop_data
            .pool
            .push(PackedUnit::pack(entity, world, resources));
        Self::do_sell(entity, resources, world);
    }

    pub fn do_sell(entity: legion::Entity, resources: &mut Resources, world: &mut legion::World) {
        Event::Sell { owner: entity }.send(world, resources);
        UnitSystem::turn_unit_into_corpse(entity, world, resources);
    }

    fn refresh_tape(world: &mut legion::World, resources: &mut Resources) {
        resources.tape_player.tape.persistent_node = Node::default().lock(NodeLockType::Factions {
            factions: Faction::all(),
            world,
            resources,
        });
    }

    pub fn floor_money(floor: usize) -> i32 {
        (3 + floor as i32).min(10)
    }

    pub fn get_g(world: &legion::World) -> i32 {
        TeamSystem::get_state(&Faction::Team, world)
            .vars
            .get_int(&VarName::G)
    }

    pub fn change_g(delta: i32, world: &mut legion::World) {
        TeamSystem::get_state_mut(&Faction::Team, world)
            .vars
            .change_int(&VarName::G, delta)
    }

    pub fn reset_g(world: &mut legion::World) {
        TeamSystem::get_state_mut(&Faction::Team, world)
            .vars
            .set_int(&VarName::G, 0);
    }

    pub fn sell_price(world: &legion::World) -> i32 {
        TeamSystem::get_state(&Faction::Team, world)
            .vars
            .get_int(&VarName::SellPrice)
    }

    pub fn buy_price(world: &legion::World) -> i32 {
        TeamSystem::get_state(&Faction::Team, world)
            .vars
            .get_int(&VarName::BuyPrice)
    }

    pub fn reroll_price(world: &legion::World, resources: &Resources) -> i32 {
        if resources.ladder.current_ind() == 0 {
            return 0;
        }
        let vars = &TeamSystem::get_state(&Faction::Team, world).vars;
        if vars.get_int(&VarName::FreeRerolls) > 0 {
            0
        } else {
            vars.get_int(&VarName::RerollPrice)
        }
    }

    fn try_reroll(world: &mut legion::World, resources: &mut Resources) {
        if Self::is_reroll_affordable(world) {
            Self::reroll(world, resources);
            Self::deduct_reroll_cost(world, resources);
        }
    }

    fn is_reroll_affordable(world: &legion::World) -> bool {
        let vars = &TeamSystem::get_state(&Faction::Team, world).vars;
        vars.try_get_int(&VarName::FreeRerolls).unwrap_or_default() > 0
            || vars.get_int(&VarName::RerollPrice) <= vars.get_int(&VarName::G)
    }

    fn deduct_reroll_cost(world: &mut legion::World, resources: &mut Resources) {
        if resources.ladder.current_ind() == 0 {
            return;
        }
        let vars = &mut TeamSystem::get_state_mut(&Faction::Team, world).vars;
        let free_rerolls = vars.try_get_int(&VarName::FreeRerolls).unwrap_or_default();
        if free_rerolls > 0 {
            vars.insert(VarName::FreeRerolls, Var::Int(free_rerolls - 1));
        } else {
            vars.change_int(&VarName::G, -vars.get_int(&VarName::RerollPrice));
        }
    }

    pub fn init_game(world: &mut legion::World, resources: &mut Resources) {
        ShopData::load_pool(resources);
        let mut team = PackedTeam::new("Sacrifice".to_owned(), default());
        team.slots = 1;
        team.unpack(&Faction::Sacrifice, world, resources);
        PackedTeam::new("Dark".to_owned(), default()).unpack(&Faction::Dark, world, resources);
        PackedTeam::new("Light".to_owned(), default()).unpack(&Faction::Light, world, resources);

        let mut team = PackedTeam::new("Shop".to_owned(), default());
        team.slots = 1;
        team.unpack(&Faction::Shop, world, resources);

        let team =
            PackedTeam::new("Team".to_owned(), default()).unpack(&Faction::Team, world, resources);
        let vars = &mut ContextState::get_mut(team, world).vars;
        vars.set_int(&VarName::G, 0);
        vars.set_int(&VarName::BuyPrice, 3);
        vars.set_int(&VarName::SellPrice, 1);
        vars.set_int(&VarName::RerollPrice, 1);
        vars.set_int(&VarName::FreeRerolls, 0);
        vars.set_int(&VarName::Slots, resources.options.initial_team_slots as i32);
        Self::create_reroll_button(world, resources);
    }

    fn reroll_btn_position(resources: &Resources) -> vec2<f32> {
        SlotSystem::get_position(0, &Faction::Shop, resources) + vec2(0.0, -2.0)
    }

    fn create_reroll_button(world: &mut legion::World, resources: &Resources) {
        fn reroll_handler(
            event: InputEvent,
            _: legion::Entity,
            _: &mut Shader,
            world: &mut legion::World,
            resources: &mut Resources,
        ) {
            match event {
                InputEvent::Click => ShopSystem::try_reroll(world, resources),
                _ => {}
            }
        }
        let entity = world.push((TapeEntityComponent {},));
        let mut button = ButtonSystem::create_button(
            Some("Reroll"),
            None,
            reroll_handler,
            entity,
            &resources.options,
        );
        button.parameters.uniforms.insert_vec_ref(
            &VarName::Position.uniform(),
            Self::reroll_btn_position(resources),
        );
        let mut entry = world.entry(entity).unwrap();
        entry.add_component(button);
        entry.add_component(EntityComponent::new(entity));
    }

    pub fn init_level(world: &mut legion::World, resources: &mut Resources, give_g: bool) {
        let current_floor = resources.ladder.current_ind();
        TeamSystem::get_state_mut(&Faction::Shop, world)
            .vars
            .set_int(
                &VarName::Slots,
                (current_floor + resources.options.initial_shop_slots).min(6) as i32,
            );
        if give_g {
            Self::change_g(Self::floor_money(current_floor), world);
        }
        TeamSystem::get_state_mut(&Faction::Shop, world)
            .vars
            .set_int(&VarName::FreeRerolls, resources.last_score as i32);

        ShopData::load_floor(resources, current_floor);
        Self::reroll(world, resources);
        WorldSystem::get_state_mut(world)
            .vars
            .set_int(&VarName::Floor, current_floor as i32);
    }

    pub fn clear_case(world: &mut legion::World, resources: &mut Resources) {
        let level = resources.ladder.current_ind();

        UnitSystem::collect_faction(world, Faction::Shop)
            .into_iter()
            .for_each(|entity| {
                if level != 0 {
                    ShopData::pack_unit_into_pool(entity, world, resources)
                } else {
                    world.remove(entity);
                }
            })
    }

    pub fn fill_case(world: &mut legion::World, resources: &mut Resources) {
        let slots = TeamSystem::get_state(&Faction::Shop, world)
            .vars
            .get_int(&VarName::Slots) as usize;
        let level = resources.ladder.current_ind();
        if level == 0 {
            let top_units = resources
                .hero_pool
                .names_sorted()
                .split_at(resources.hero_pool.len() - 10)
                .1
                .into_iter()
                .cloned()
                .choose_multiple(&mut thread_rng(), slots);
            let team = TeamSystem::entity(&Faction::Shop, world).unwrap();
            for (slot, name) in top_units.into_iter().enumerate() {
                let slot = slot + 1;
                let unit = resources.hero_pool.find_by_name(&name).unwrap().clone();
                unit.unpack(world, resources, slot, None, team);
            }
        } else {
            for slot in 1..=slots {
                let pool_len = ShopData::pool_len(resources);
                if pool_len == 0 {
                    return;
                }
                let mut rng = rand::thread_rng();
                let ind = rng.gen_range(0..pool_len);
                ShopData::unpack_pool_unit(ind, slot, resources, world);
            }
        }
    }

    pub fn reroll(world: &mut legion::World, resources: &mut Resources) {
        Self::clear_case(world, resources);
        Self::fill_case(world, resources);
    }
}
