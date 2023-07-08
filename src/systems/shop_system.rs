use crate::resources::Widget;

use super::*;

#[derive(Default)]
pub struct ShopSystem;

#[derive(enum_iterator::Sequence, Clone, Copy)]
pub enum Product {
    Hero,
    Buff,
    AoeBuff,
    TeamBuff,
    Slot,
}

impl Product {
    pub fn index(&self) -> usize {
        *self as usize
    }

    pub fn price(&self) -> usize {
        match self {
            Product::Hero => 3,
            Product::Buff => 2,
            Product::AoeBuff => 5,
            Product::TeamBuff => 9,
            Product::Slot => 4,
        }
    }

    pub fn input_handler(&self) -> Handler {
        match self {
            Product::Hero => {
                fn input_handler(
                    event: HandleEvent,
                    entity: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    match event {
                        HandleEvent::Click => {
                            let price = shader.get_int("u_price");
                            if ShopSystem::get_g(world) >= price
                                && resources
                                    .tape_player
                                    .tape
                                    .close_panels(entity, resources.tape_player.head)
                            {
                                ShopSystem::change_g(-price, Some("Buy Hero"), world, resources);
                                ShopSystem::show_hero_buy_panel(resources);
                            }
                        }
                        _ => {}
                    }
                }
                input_handler
            }
            Product::Buff => {
                fn input_handler(
                    event: HandleEvent,
                    entity: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    match event {
                        HandleEvent::Click => {
                            let price = shader.get_int("u_price");
                            if ShopSystem::get_g(world) >= price
                                && UnitSystem::collect_faction(world, Faction::Team).len() > 0
                                && resources
                                    .tape_player
                                    .tape
                                    .close_panels(entity, resources.tape_player.head)
                            {
                                ShopSystem::change_g(-price, Some("Buy Buff"), world, resources);
                                ShopSystem::show_buff_buy_panel(resources);
                            }
                        }
                        _ => {}
                    }
                }
                input_handler
            }
            Product::AoeBuff => {
                fn input_handler(
                    event: HandleEvent,
                    entity: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    match event {
                        HandleEvent::Click => {
                            let price = shader.get_int("u_price");
                            if ShopSystem::get_g(world) >= price
                                && UnitSystem::collect_faction(world, Faction::Team).len() > 0
                                && resources
                                    .tape_player
                                    .tape
                                    .close_panels(entity, resources.tape_player.head)
                            {
                                ShopSystem::change_g(
                                    -price,
                                    Some("Buy Aoe Buff"),
                                    world,
                                    resources,
                                );
                                ShopSystem::show_aoe_buff_buy_panel(resources);
                            }
                        }
                        _ => {}
                    }
                }
                input_handler
            }
            Product::TeamBuff => {
                fn input_handler(
                    event: HandleEvent,
                    entity: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    match event {
                        HandleEvent::Click => {
                            let price = shader.get_int("u_price");
                            if ShopSystem::get_g(world) >= price
                                && resources
                                    .tape_player
                                    .tape
                                    .close_panels(entity, resources.tape_player.head)
                            {
                                ShopSystem::change_g(
                                    -price,
                                    Some("Buy Team Buff"),
                                    world,
                                    resources,
                                );
                                ShopSystem::show_team_buff_buy_panel(resources);
                            }
                        }
                        _ => {}
                    }
                }
                input_handler
            }
            Product::Slot => {
                fn input_handler(
                    event: HandleEvent,
                    entity: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    resources: &mut Resources,
                ) {
                    match event {
                        HandleEvent::Click => {
                            let price = shader.get_int("u_price");
                            if ShopSystem::get_g(world) >= price
                                && resources
                                    .tape_player
                                    .tape
                                    .close_panels(entity, resources.tape_player.head)
                            {
                                ShopSystem::change_g(-price, Some("Buy Slot"), world, resources);
                                TeamSystem::change_slots(1, Faction::Team, world);
                                if TeamSystem::get_state(Faction::Team, world)
                                    .vars
                                    .get_int(&VarName::Slots)
                                    < MAX_SLOTS as i32
                                {
                                    Product::Slot.create_button(resources);
                                }
                                PanelsSystem::open_push(
                                    resources.options.colors.add,
                                    "Add Slot",
                                    "+1 Slot",
                                    resources,
                                );
                            }
                        }
                        _ => {}
                    }
                }
                input_handler
            }
        }
    }

    pub fn update_handler(&self) -> Handler {
        match self {
            Product::Hero | Product::TeamBuff | Product::Slot => {
                fn update_handler(
                    _: HandleEvent,
                    _: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    _: &mut Resources,
                ) {
                    shader.set_active(
                        ShopSystem::get_g(world)
                            >= shader.parameters.uniforms.try_get_int("u_price").unwrap(),
                    );
                }
                update_handler
            }
            Product::Buff | Product::AoeBuff => {
                fn update_handler(
                    _: HandleEvent,
                    _: legion::Entity,
                    shader: &mut Shader,
                    world: &mut legion::World,
                    _: &mut Resources,
                ) {
                    let team_empty = UnitSystem::collect_faction(world, Faction::Team).len() == 0;
                    shader.set_active(
                        ShopSystem::get_g(world)
                            >= shader.parameters.uniforms.try_get_int("u_price").unwrap()
                            && !team_empty,
                    );
                }
                update_handler
            }
        }
    }

    pub fn hover_hints(&self, cost: usize, options: &Options) -> Vec<(Rgba<f32>, String, String)> {
        match self {
            Product::Hero => vec![(
                options.colors.shop,
                "Buy Hero".to_owned(),
                format!("-{cost} g"),
            )],
            Product::Buff => vec![(
                options.colors.shop,
                "Buy Buff".to_owned(),
                format!("Add buff to 1 hero\n-{cost} g"),
            )],
            Product::AoeBuff => vec![(
                options.colors.shop,
                "Buy Aoe Buff".to_owned(),
                format!("Apply a buff\nto each hero on the team\n-{cost} g"),
            )],
            Product::TeamBuff => vec![(
                options.colors.shop,
                "Buy Team Buff".to_owned(),
                format!("Add buff that is always\napplied to whole team\n-{cost} g"),
            )],
            Product::Slot => vec![(
                options.colors.shop,
                "Buy Slot".to_owned(),
                format!("+1 team slot\n-{cost} g"),
            )],
        }
    }

    pub fn title(&self) -> String {
        match self {
            Product::Hero => "Buy Hero".to_owned(),
            Product::Buff => "Buy Buff".to_owned(),
            Product::AoeBuff => "Buy Aoe Buff".to_owned(),
            Product::TeamBuff => "Buy Team Buff".to_owned(),
            Product::Slot => "Buy Slot".to_owned(),
        }
    }

    pub fn create_button(&self, resources: &mut Resources) {
        let entity = new_entity();
        let price = self.price();
        let hover_hints = self.hover_hints(price, &resources.options);
        let title = self.title();
        Widget::Button {
            text: title,
            input_handler: self.input_handler(),
            update_handler: Some(self.update_handler()),
            options: &resources.options,
            uniforms: resources
                .options
                .uniforms
                .ui_button
                .clone()
                .insert_int("u_index".to_owned(), self.index() as i32)
                .insert_int("u_price".to_owned(), price as i32),
            shader: None,
            entity,
            hover_hints,
        }
        .generate_node()
        .lock(NodeLockType::Empty)
        .push_as_panel(entity, resources);
    }
}

impl System for ShopSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        Self::refresh_tape(world, resources);
        if !resources.action_queue.is_empty() {
            let mut cluster = Some(NodeCluster::default());
            ActionSystem::run_ticks(world, resources, cluster.as_mut());
            ActionSystem::death_check(world, resources, cluster.as_mut());
            ActionSystem::run_ticks(world, resources, cluster.as_mut());

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

    pub fn show_hero_buy_panel(resources: &mut Resources) {
        // let mut units = Vec::default();
        // for _ in 0..3 {
        //     let pool = &mut resources.shop_data.pool;
        //     let unit = (0..pool.len()).choose(&mut thread_rng()).unwrap();
        //     let unit = pool.swap_remove(unit);
        //     units.push(unit);
        // }
        let units = resources
            .shop_data
            .pool
            .choose_multiple_weighted(&mut thread_rng(), 3, |x| {
                HeroPool::rarity_by_name(&x.name, resources).weight()
            })
            .unwrap()
            .cloned()
            .collect_vec();
        let choice = CardChoice::BuyHero { units };
        PanelsSystem::open_card_choice(choice, resources);
    }

    pub fn show_buff_buy_panel(resources: &mut Resources) {
        let choice = CardChoice::BuyBuff {
            buffs: BuffPool::get_random(3, resources),
            target: BuffTarget::Single { slot: None },
        };
        PanelsSystem::open_card_choice(choice, resources);
    }

    pub fn show_team_buff_buy_panel(resources: &mut Resources) {
        let choice = CardChoice::BuyBuff {
            buffs: BuffPool::get_random(3, resources),
            target: BuffTarget::Team,
        };
        PanelsSystem::open_card_choice(choice, resources);
    }

    pub fn show_aoe_buff_buy_panel(resources: &mut Resources) {
        let choice = CardChoice::BuyBuff {
            buffs: BuffPool::get_random(3, resources),
            target: BuffTarget::Aoe,
        };
        PanelsSystem::open_card_choice(choice, resources);
    }

    pub fn show_battle_choice_panel(resources: &mut Resources) {
        let teams = Ladder::generate_current_teams(resources)
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
        let team = TeamSystem::entity(Faction::Team, world);
        unit.unpack(world, resources, 0, None, team);
        SlotSystem::fill_gaps(Faction::Team, world);
    }

    pub fn do_buy(
        entity: legion::Entity,
        slot: usize,
        resources: &mut Resources,
        world: &mut legion::World,
    ) {
        let team = TeamSystem::entity(Faction::Team, world);
        let state = ContextState::get_mut(entity, world);
        state.parent = team;
        state.vars.set_int(&VarName::Slot, slot as i32);

        Event::Buy { owner: entity }.send(world, resources);
        Event::AddToTeam { owner: entity }.send(world, resources);
    }

    fn refresh_tape(world: &mut legion::World, resources: &mut Resources) {
        resources.tape_player.tape.persistent_node = Node::default().lock(NodeLockType::Factions {
            factions: Faction::all(),
            world,
            resources,
        });
    }

    pub fn get_g(world: &legion::World) -> i32 {
        TeamSystem::get_state(Faction::Team, world)
            .vars
            .get_int(&VarName::G)
    }

    pub fn is_just_started(world: &legion::World, resources: &Resources) -> bool {
        Ladder::current_level(resources) == 0
            && UnitSystem::collect_faction(world, Faction::Team).is_empty()
    }

    pub fn change_g(
        delta: i32,
        reason: Option<&str>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        TeamSystem::get_state_mut(Faction::Team, world)
            .vars
            .change_int(&VarName::G, delta);
        PanelsSystem::refresh_stats(world, resources);
        if let Some(reason) = reason {
            let sign = match delta.signum() > 0 {
                true => "+",
                false => "",
            };
            PanelsSystem::open_push(
                resources.options.colors.shop,
                reason,
                &format!("{sign}{delta} g"),
                resources,
            );
        }
    }

    pub fn reset_g(world: &mut legion::World) {
        TeamSystem::get_state_mut(Faction::Team, world)
            .vars
            .set_int(&VarName::G, 0);
    }

    pub fn is_reroll_affordable(world: &legion::World) -> bool {
        let vars = &TeamSystem::get_state(Faction::Team, world).vars;
        vars.try_get_int(&VarName::FreeRerolls).unwrap_or_default() > 0
            || vars.get_int(&VarName::RerollPrice) <= vars.get_int(&VarName::G)
    }

    pub fn deduct_reroll_cost(world: &mut legion::World, resources: &mut Resources) {
        if Self::is_just_started(world, resources) {
            return;
        }
        let vars = &mut TeamSystem::get_state_mut(Faction::Team, world).vars;
        let free_rerolls = vars.try_get_int(&VarName::FreeRerolls).unwrap_or_default();
        if free_rerolls > 0 {
            vars.change_int(&VarName::FreeRerolls, -1);
        } else {
            Self::change_g(
                -vars.get_int(&VarName::RerollPrice),
                Some("Reroll"),
                world,
                resources,
            );
        }
    }

    pub fn init_game(world: &mut legion::World, resources: &mut Resources) {
        ShopData::load_pool_full(resources);
        PackedTeam::new("Dark".to_owned(), default()).unpack(&Faction::Dark, world, resources);
        PackedTeam::new("Light".to_owned(), default()).unpack(&Faction::Light, world, resources);
        PackedTeam::new("Team".to_owned(), default()).unpack(&Faction::Team, world, resources);

        let vars = &mut TeamSystem::get_state_mut(Faction::Team, world).vars;
        vars.set_int(&VarName::G, 0);
        vars.set_int(&VarName::BuyPrice, 3);
        vars.set_int(&VarName::SellPrice, 1);
        vars.set_int(&VarName::RerollPrice, 1);
        vars.set_int(&VarName::FreeRerolls, 0);
        vars.set_int(&VarName::Slots, resources.options.initial_team_slots as i32);
    }

    pub fn enter(world: &mut legion::World, resources: &mut Resources) {
        let current_level = Ladder::current_level(resources);
        if current_level == 0 {
            Self::change_g(resources.options.initial_shop_g, None, world, resources);
        }
        ShopData::load_floor(resources, current_level);
        WorldSystem::get_state_mut(world)
            .vars
            .set_int(&VarName::Level, current_level as i32);
        Self::create_battle_button(resources);

        for product in enum_iterator::all::<Product>() {
            product.create_button(resources);
        }
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
        let hover_hints = vec![(
            resources.options.colors.enemy,
            "Start Battle".to_owned(),
            format!("Choose enemy by difficulty\nand send copy of team\ninto battle"),
        )];
        let uniforms = resources
            .options
            .uniforms
            .ui_button
            .clone()
            .insert_int("u_index".to_owned(), -1);
        Widget::Button {
            text: "Start Battle".to_owned(),
            input_handler,
            update_handler: Some(update_handler),
            options: &resources.options,
            uniforms,
            shader: None,
            hover_hints,
            entity,
        }
        .generate_node()
        .lock(NodeLockType::Empty)
        .push_as_panel(entity, resources);
    }

    pub fn start_buff_apply(
        name: String,
        charges: i32,
        target: BuffTarget,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        resources.shop_data.status_apply = Some((name, charges, target));
        match &target {
            BuffTarget::Single { .. } => {
                SlotSystem::add_slots_buttons(
                    Faction::Team,
                    "Apply",
                    Some("u_filled"),
                    None,
                    world,
                    resources,
                );
            }
            BuffTarget::Aoe | BuffTarget::Team => Self::finish_buff_apply(world, resources),
        }
    }

    pub fn finish_buff_apply(world: &mut legion::World, resources: &mut Resources) {
        let (name, charges, target) = resources.shop_data.status_apply.take().unwrap();
        let mut node = Some(Node::default());
        let mut entities = Vec::default();
        match target {
            BuffTarget::Single { slot } => {
                entities.push(
                    SlotSystem::find_unit_by_slot(
                        slot.expect("Slot wasn't set for buff apply"),
                        &Faction::Team,
                        world,
                    )
                    .unwrap(),
                );
                Product::Buff.create_button(resources);
            }
            BuffTarget::Aoe => {
                for unit in UnitSystem::collect_faction(world, Faction::Team) {
                    entities.push(unit);
                }
                Product::AoeBuff.create_button(resources);
            }
            BuffTarget::Team => {
                entities.push(TeamSystem::entity(Faction::Team, world).unwrap());
                PanelsSystem::open_push(
                    resources.options.colors.player,
                    "New Team Status",
                    &format!("{name} +{charges}"),
                    resources,
                );
                Product::TeamBuff.create_button(resources);
            }
        };
        for entity in entities {
            Status::change_charges(entity, charges, &name, &mut node, world, resources);
        }
        PanelsSystem::refresh_stats(world, resources);
        resources.tape_player.tape.push_to_queue(
            NodeCluster::new(node.unwrap().lock(NodeLockType::Empty)),
            resources.tape_player.head,
        );
        SlotSystem::clear_slots_buttons(Faction::Team, world);
    }
}
