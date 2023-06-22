use crate::resources::Widget;

use super::*;

#[derive(Default)]
pub struct ShopSystem {
    need_switch_battle: bool,
}

impl System for ShopSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if self.need_switch_battle {
            Self::show_battle_choice_panel(resources);
            self.need_switch_battle = false;
        }
        Self::refresh_tape(world, resources);
        if !resources.action_queue.is_empty() {
            let mut cluster = Some(NodeCluster::default());
            ActionSystem::run_ticks(world, resources, &mut cluster);
            ActionSystem::death_check(world, resources, &mut cluster);
            ActionSystem::run_ticks(world, resources, &mut cluster);

            resources
                .tape_player
                .tape
                .push_to_queue(cluster.unwrap(), resources.tape_player.head);
        }
    }
}

impl ShopSystem {
    pub fn new() -> Self {
        default()
    }

    pub fn reroll_hero_panel(world: &mut legion::World, resources: &mut Resources) {
        Self::return_showed_heroes(resources);
        Self::show_hero_buy_panel(resources);
        Self::deduct_reroll_cost(world, resources);
    }

    fn return_showed_heroes(resources: &mut Resources) {
        resources
            .shop_data
            .offered
            .drain(..)
            .for_each(|x| resources.shop_data.pool.push(x));
    }

    pub fn show_hero_buy_panel(resources: &mut Resources) {
        let mut units = Vec::default();
        for _ in 0..3 {
            let pool = &mut resources.shop_data.pool;
            let unit = (0..pool.len()).choose(&mut thread_rng()).unwrap();
            let unit = pool.swap_remove(unit);
            units.push(unit);
        }
        let choice = CardChoice::BuyHero { units };
        PanelsSystem::open_card_choice(choice, resources);
    }

    pub fn show_status_buy_panel(resources: &mut Resources) {
        let mut statuses = Vec::default();
        statuses.push(("Chaotic".to_owned(), 2));
        statuses.push(("Fortitude".to_owned(), 3));
        statuses.push(("Shield".to_owned(), 1));
        let choice = CardChoice::BuyStatus { statuses };
        PanelsSystem::open_card_choice(choice, resources);
    }

    pub fn show_battle_choice_panel(resources: &mut Resources) {
        let teams = Ladder::get_current_teams(resources)
            .into_iter()
            .map(|x| x.clone())
            .collect_vec();
        let choice = CardChoice::SelectEnemy { teams };
        PanelsSystem::open_card_choice(choice, resources);
    }

    pub fn add_unit_to_team(
        unit: PackedUnit,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        let team = TeamSystem::entity(&Faction::Team, world);
        unit.unpack(world, resources, 0, None, team);
        SlotSystem::fill_gaps(Faction::Team, world);
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
        let price = Self::sell_price(world);
        let color = Faction::Shop.color(&resources.options);
        let position = Context::new(ContextLayer::Entity { entity }, world, resources)
            .get_vec2(&VarName::Position, world)
            .unwrap();
        VfxSystem::add_show_text_effect(&format!("+{price} g"), color, position, world, resources);
        Self::change_g(price, Some("Sell hero"), world, resources);
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

    pub fn get_g(world: &legion::World) -> i32 {
        TeamSystem::get_state(&Faction::Team, world)
            .vars
            .get_int(&VarName::G)
    }

    pub fn change_g(
        delta: i32,
        reason: Option<&str>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        TeamSystem::get_state_mut(&Faction::Team, world)
            .vars
            .change_int(&VarName::G, delta);
        PanelsSystem::refresh_stats(world, resources);
        if let Some(reason) = reason {
            let sign = match delta.signum() > 0 {
                true => "+",
                false => "",
            };
            PanelsSystem::open_push(
                resources
                    .options
                    .colors
                    .factions
                    .get(&Faction::Shop)
                    .unwrap()
                    .clone(),
                reason,
                &format!("{sign}{delta} g"),
                resources,
            );
        }
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

    pub fn is_reroll_affordable(world: &legion::World) -> bool {
        let vars = &TeamSystem::get_state(&Faction::Team, world).vars;
        vars.try_get_int(&VarName::FreeRerolls).unwrap_or_default() > 0
            || vars.get_int(&VarName::RerollPrice) <= vars.get_int(&VarName::G)
    }

    pub fn is_hero_affordable(world: &legion::World) -> bool {
        Self::get_g(world) >= Self::buy_price(world)
    }

    pub fn deduct_reroll_cost(world: &mut legion::World, resources: &mut Resources) {
        if resources.ladder.current_ind() == 0
            && UnitSystem::collect_faction(world, Faction::Team).len() == 0
        {
            return;
        }
        let vars = &mut TeamSystem::get_state_mut(&Faction::Team, world).vars;
        let free_rerolls = vars.try_get_int(&VarName::FreeRerolls).unwrap_or_default();
        if free_rerolls > 0 {
            vars.change_int(&VarName::FreeRerolls, -1);
        } else {
            vars.change_int(&VarName::G, -vars.get_int(&VarName::RerollPrice));
        }
    }

    pub fn deduct_hero_price(world: &mut legion::World, resources: &mut Resources) {
        let price = Self::buy_price(world);
        Self::change_g(-price, Some("Buy hero"), world, resources);
    }

    pub fn init_game(world: &mut legion::World, resources: &mut Resources) {
        ShopData::load_pool_full(resources);
        PackedTeam::new("Dark".to_owned(), default()).unpack(&Faction::Dark, world, resources);
        PackedTeam::new("Light".to_owned(), default()).unpack(&Faction::Light, world, resources);
        PackedTeam::new("Team".to_owned(), default()).unpack(&Faction::Team, world, resources);

        let vars = &mut TeamSystem::get_state_mut(&Faction::Team, world).vars;
        vars.set_int(&VarName::G, 0);
        vars.set_int(&VarName::BuyPrice, 3);
        vars.set_int(&VarName::SellPrice, 1);
        vars.set_int(&VarName::RerollPrice, 1);
        vars.set_int(&VarName::FreeRerolls, 0);
        vars.set_int(&VarName::Slots, resources.options.initial_team_slots as i32);
    }

    pub fn enter(world: &mut legion::World, resources: &mut Resources) {
        let current_floor = resources.ladder.current_ind();
        if current_floor == 0 {
            Self::change_g(resources.options.initial_shop_g, None, world, resources);
        }
        ShopData::load_floor(resources, current_floor);
        WorldSystem::get_state_mut(world)
            .vars
            .set_int(&VarName::Level, current_floor as i32);
        Self::create_buy_hero_button(resources);
        Self::create_buy_status_button(resources);
        Self::create_battle_button(resources);
    }

    pub fn leave(world: &mut legion::World, resources: &mut Resources) {
        resources.tape_player.clear();
        Event::ShopEnd.send(world, resources);
        ShopSystem::reset_g(world);
    }

    fn create_battle_button(resources: &mut Resources) {
        fn input_handler(
            event: HandleEvent,
            entity: legion::Entity,
            _: &mut Shader,
            world: &mut legion::World,
            resources: &mut Resources,
        ) {
            match event {
                HandleEvent::Click => {
                    if !UnitSystem::collect_faction(world, Faction::Team).is_empty()
                        && resources
                            .tape_player
                            .tape
                            .close_panels(entity, resources.tape_player.head)
                    {
                        ShopSystem::show_battle_choice_panel(resources);
                    }
                }
                _ => {}
            }
        }
        fn update_handler(
            _: HandleEvent,
            _: legion::Entity,
            shader: &mut Shader,
            world: &mut legion::World,
            _: &mut Resources,
        ) {
            shader.set_active(!UnitSystem::collect_faction(world, Faction::Team).is_empty());
        }
        let entity = new_entity();
        let uniforms = resources
            .options
            .uniforms
            .ui_button
            .clone()
            .insert_int("u_index".to_owned(), 1);
        Widget::Button {
            text: "Start battle".to_owned(),
            input_handler,
            update_handler: Some(update_handler),
            options: &resources.options,
            uniforms,
            shader: None,
            hover_hints: default(),
            entity,
        }
        .generate_node()
        .lock(NodeLockType::Empty)
        .push_as_panel(entity, resources);
    }

    pub fn create_buy_hero_button(resources: &mut Resources) {
        fn input_handler(
            event: HandleEvent,
            entity: legion::Entity,
            _: &mut Shader,
            world: &mut legion::World,
            resources: &mut Resources,
        ) {
            match event {
                HandleEvent::Click => {
                    if ShopSystem::is_hero_affordable(world)
                        && resources
                            .tape_player
                            .tape
                            .close_panels(entity, resources.tape_player.head)
                    {
                        ShopSystem::deduct_hero_price(world, resources);
                        ShopSystem::show_hero_buy_panel(resources);
                    }
                }
                _ => {}
            }
        }
        fn update_handler(
            _: HandleEvent,
            _: legion::Entity,
            shader: &mut Shader,
            world: &mut legion::World,
            _: &mut Resources,
        ) {
            shader.set_active(ShopSystem::is_hero_affordable(world));
        }
        let entity = new_entity();
        let hover_hints = vec![];
        Widget::Button {
            text: "Buy hero".to_owned(),
            input_handler,
            update_handler: Some(update_handler),
            options: &resources.options,
            uniforms: resources.options.uniforms.ui_button.clone(),
            shader: None,
            hover_hints,
            entity,
        }
        .generate_node()
        .lock(NodeLockType::Empty)
        .push_as_panel(entity, resources);
    }

    pub fn create_buy_status_button(resources: &mut Resources) {
        fn input_handler(
            event: HandleEvent,
            entity: legion::Entity,
            _: &mut Shader,
            world: &mut legion::World,
            resources: &mut Resources,
        ) {
            match event {
                HandleEvent::Click => {
                    if ShopSystem::is_hero_affordable(world)
                        && resources
                            .tape_player
                            .tape
                            .close_panels(entity, resources.tape_player.head)
                    {
                        ShopSystem::deduct_hero_price(world, resources);
                        ShopSystem::show_status_buy_panel(resources);
                    }
                }
                _ => {}
            }
        }
        fn update_handler(
            _: HandleEvent,
            _: legion::Entity,
            shader: &mut Shader,
            world: &mut legion::World,
            _: &mut Resources,
        ) {
            let team_empty = UnitSystem::collect_faction(world, Faction::Team).len() == 0;
            shader.set_active(ShopSystem::is_hero_affordable(world) && !team_empty);
        }
        let entity = new_entity();
        Widget::Button {
            text: "Buy status".to_owned(),
            input_handler,
            update_handler: Some(update_handler),
            options: &resources.options,
            uniforms: resources
                .options
                .uniforms
                .ui_button
                .clone()
                .insert_int("u_index".to_owned(), -1),
            shader: None,
            entity,
            hover_hints: default(),
        }
        .generate_node()
        .lock(NodeLockType::Empty)
        .push_as_panel(entity, resources);
    }

    pub fn start_status_apply(
        name: String,
        charges: i32,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        SlotSystem::add_slots_buttons(
            Faction::Team,
            "Apply",
            Some("u_filled"),
            None,
            world,
            resources,
        );
        resources.shop_data.status_apply = Some((name, charges));
    }

    pub fn finish_status_apply(slot: usize, world: &mut legion::World, resources: &mut Resources) {
        let (name, charges) = resources.shop_data.status_apply.take().unwrap();
        Status::change_charges(
            SlotSystem::find_unit_by_slot(slot, &Faction::Team, world).unwrap(),
            charges,
            &name,
            &mut None,
            world,
            resources,
        );
        SlotSystem::clear_slots_buttons(Faction::Team, world);
        Self::create_buy_status_button(resources);
    }
}
