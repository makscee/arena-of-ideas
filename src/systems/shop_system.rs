use crate::resources::Widget;

use super::*;

#[derive(Default)]
pub struct ShopSystem;

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

    pub fn show_offers_panel(resources: &mut Resources) {
        let units = resources
            .shop_data
            .pool
            .choose_multiple_weighted(&mut thread_rng(), 3, |x| {
                HeroPool::rarity_by_name(&x.name, resources).weight()
            })
            .unwrap()
            .cloned()
            .collect_vec();
        let buffs = BuffPool::get_random(2, resources)
            .into_iter()
            .map(|x| (x, BuffTarget::random()))
            .collect_vec();
        let choice = CardChoice::ShopOffers { units, buffs };
        PanelsSystem::open_card_choice(choice, resources);
        resources.panels_data.removed_inds = default();
        debug!("Show offers");
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

    pub fn reroll_cost(world: &legion::World, resources: &Resources) -> usize {
        if Self::is_just_started(world, resources) {
            return 0;
        }
        let vars = &TeamSystem::get_state(Faction::Team, world).vars;
        let free_rerolls = vars.try_get_int(&VarName::FreeRerolls).unwrap_or_default();
        if free_rerolls > 0 {
            return 0;
        } else {
            return vars.get_int(&VarName::RerollPrice) as usize;
        }
    }

    pub fn deduct_reroll_cost(world: &mut legion::World, resources: &mut Resources) {
        let cost = Self::reroll_cost(world, resources);
        if cost > 0 {
            Self::change_g(-(cost as i32), Some("Reroll"), world, resources);
        } else {
            let vars = &mut TeamSystem::get_state_mut(Faction::Team, world).vars;
            let free_rerolls = vars.try_get_int(&VarName::FreeRerolls).unwrap_or_default();
            if free_rerolls > 0 {
                vars.change_int(&VarName::FreeRerolls, -1);
            }
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

    pub fn level_g(resources: &Resources) -> usize {
        (resources.options.start_g + Ladder::current_level(resources)).min(resources.options.max_g)
    }

    pub fn enter(world: &mut legion::World, resources: &mut Resources) {
        let level = Ladder::current_level(resources);
        ShopData::load_level(resources, level);
        Self::change_g(
            Self::level_g(resources) as i32,
            Some(&format!("Level {} start", level + 1)),
            world,
            resources,
        );
        WorldSystem::get_state_mut(world)
            .vars
            .set_int(&VarName::Level, level as i32);
        Self::create_battle_button(resources);
        Self::show_offers_panel(resources);
    }

    fn is_max_slots(world: &legion::World) -> bool {
        TeamSystem::get_state(Faction::Team, world)
            .vars
            .get_int(&VarName::Slots)
            >= MAX_SLOTS as i32
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
                        PanelsSystem::close_all_alerts(resources);
                        resources.panels_data.choice_options = None;
                        Ladder::start_next_battle(world, resources);
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
            }
            BuffTarget::Aoe => {
                for unit in UnitSystem::collect_faction(world, Faction::Team) {
                    entities.push(unit);
                }
            }
            BuffTarget::Team => {
                entities.push(TeamSystem::entity(Faction::Team, world).unwrap());
                PanelsSystem::open_push(
                    resources.options.colors.player,
                    "New Team Status",
                    &format!("{name} +{charges}"),
                    resources,
                );
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
