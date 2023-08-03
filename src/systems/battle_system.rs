use super::*;

pub struct BattleSystem {}

impl BattleSystem {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run_battle(
        world: &mut legion::World,
        resources: &mut Resources,
        tape: &mut Option<Tape>,
    ) -> usize {
        let mut cluster = match tape {
            Some(_) => Some(NodeCluster::default()),
            None => None,
        };
        SlotSystem::move_to_slots_animated(world, resources, cluster.as_mut());
        Event::BattleStart.send(world, resources);
        ActionSystem::spin(world, resources, cluster.as_mut());
        if let Some(tape) = tape {
            tape.push(cluster.unwrap());
        }
        let mut turns = 0;
        loop {
            turns += 1;
            resources.battle_data.turns = turns;
            if turns > 100 {
                error!(
                    "Exceeded turns limit: {turns}, {} x {}",
                    TeamSystem::get_state(Faction::Light, world).name,
                    TeamSystem::get_state(Faction::Dark, world).name
                );
                return 0;
            }
            // if turns % 10 == 0 {
            //     Self::promote_all(world);
            // }
            let result = Self::turn(world, resources, tape);
            if !result {
                break;
            }
        }
        Ladder::get_score(world)
    }

    pub fn init_battle(
        light: &PackedTeam,
        dark: &PackedTeam,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        Self::clear_world(world, resources);
        light.unpack(Faction::Light, world, resources);
        dark.unpack(Faction::Dark, world, resources);
        resources.battle_data.turns = 0;
    }

    pub fn enter_state(world: &mut legion::World, resources: &mut Resources) {
        resources.camera.focus = Focus::Battle;
        resources.tape_player.clear();
        Self::open_curses_panel(resources);
    }

    pub fn open_curses_panel(resources: &mut Resources) {
        resources.battle_data.curse_choice = vec![
            CursePool::get_random(resources),
            CursePool::get_random(resources),
        ];
        resources.battle_data.applied_curses.clear();

        fn update_curse(
            _: HandleEvent,
            _: legion::Entity,
            shader: &mut Shader,
            _: &mut legion::World,
            resources: &mut Resources,
        ) {
            let ind = shader.get_int("u_index") as usize;
            if resources.battle_data.applied_curses.contains(&ind) {
                shader.set_enabled(false);
            }
        }

        fn apply_curse(
            event: HandleEvent,
            _: legion::Entity,
            shader: &mut Shader,
            _: &mut legion::World,
            resources: &mut Resources,
        ) {
            match event {
                HandleEvent::Click => {
                    let ind = shader.get_int("u_index") as usize;
                    resources.action_queue.push_front(Action::new(
                        Context::new_empty("Apply Curse"),
                        resources.battle_data.curse_choice[ind]
                            .positive_effect
                            .clone(),
                    ));
                    resources.action_queue.push_front(Action::new(
                        Context::new_empty("Apply Curse"),
                        resources.battle_data.curse_choice[ind]
                            .negative_effect
                            .clone(),
                    ));
                    resources.battle_data.applied_curses.insert(ind);
                }
                _ => {}
            }
        }
        let button = PanelFooterButton::Custom {
            name: "Apply".to_owned(),
            handler: apply_curse,
        };
        let shaders = (0..2)
            .into_iter()
            .map(|ind| {
                let curse = &resources.battle_data.curse_choice[ind];
                let pos_text = curse.positive_text.clone();
                let neg_text = curse.negative_text.clone();
                let mut shader = resources
                    .options
                    .shaders
                    .battle_curse
                    .clone()
                    .wrap_panel_body(vec2::ZERO, &resources.options)
                    .insert_int("u_index".to_owned(), ind as i32)
                    .insert_string("u_positive_text".to_owned(), pos_text, 1)
                    .insert_string("u_negative_text".to_owned(), neg_text, 1)
                    .wrap_panel_header("Curse", &resources.options)
                    .wrap_panel_footer(vec![button.clone()], &resources.options);
                shader.middle.pre_update_handlers.push(update_curse);
                shader
            })
            .collect_vec();
        let padding = resources.options.floats.panel_row_padding;
        let shader =
            ShaderChain::wrap_panel_body_row(shaders, vec2(padding, padding), &resources.options);
        PanelsSystem::add_alert(
            resources.options.colors.enemy,
            &format!(
                "Level {}/{}",
                Ladder::current_ind(resources) + 1,
                Ladder::count(resources)
            ),
            shader,
            vec2::ZERO,
            vec![PanelFooterButton::Start],
            resources,
        );
    }

    pub fn play_battle(world: &mut legion::World, resources: &mut Resources) {
        let mut tape = Some(Tape::default());
        resources.battle_data.last_score = BattleSystem::run_battle(world, resources, &mut tape);
        resources.tape_player.clear();
        resources.tape_player.tape = tape.unwrap();
        resources.tape_player.mode = TapePlayMode::Play;
    }

    pub fn finish_ladder_battle(world: &mut legion::World, resources: &mut Resources) {
        resources.tape_player.clear();
        resources.battle_data.last_score = Ladder::get_score(world);
        resources.battle_data.total_score += resources.battle_data.last_score;
        let level = Ladder::current_ind(resources) + 1;
        let (title, text, buttons, color) = if resources.battle_data.last_score > 0 {
            fn input_handler(
                event: HandleEvent,
                _: legion::Entity,
                _: &mut Shader,
                _: &mut legion::World,
                resources: &mut Resources,
            ) {
                match event {
                    HandleEvent::Click => {
                        resources.transition_state = GameState::Shop;
                    }
                    _ => {}
                }
            }
            let buttons = vec![PanelFooterButton::Custom {
                name: "Continue".to_owned(),
                handler: input_handler,
            }];
            if Ladder::next(resources) {
                (
                    "Victory",
                    format!("Level {level} complete!"),
                    buttons,
                    resources.options.colors.victory,
                )
            } else {
                (
                    "Victory",
                    format!("All {level} levels complete!\nNew level will be added to ladder."),
                    buttons,
                    resources.options.colors.victory,
                )
            }
        } else {
            (
                "Defeat",
                format!(
                    "Game Over\n{} levels complete",
                    Ladder::current_ind(resources)
                ),
                vec![PanelFooterButton::Restart],
                resources.options.colors.defeat,
            )
        };
        PanelsSystem::add_text_alert(color, title, &text, vec2(0.0, 0.3), buttons, resources);
    }

    pub fn clear_world(world: &mut legion::World, resources: &mut Resources) {
        UnitSystem::clear_factions(&Faction::battle(), world);
    }

    fn strickers_death_check(
        left: legion::Entity,
        right: legion::Entity,
        world: &legion::World,
    ) -> bool {
        UnitSystem::get_corpse(left, world).is_some()
            || UnitSystem::get_corpse(right, world).is_some()
    }

    pub fn turn(
        world: &mut legion::World,
        resources: &mut Resources,
        tape: &mut Option<Tape>,
    ) -> bool {
        for faction in Faction::battle() {
            SlotSystem::fill_gaps(faction, world);
        }
        let mut cluster = match tape {
            Some(_) => Some(NodeCluster::default()),
            None => None,
        };
        ActionSystem::spin(world, resources, cluster.as_mut());
        SlotSystem::move_to_slots_animated(world, resources, cluster.as_mut());
        if let Some((left, right)) = Self::find_hitters(world) {
            Event::TurnStart.send(world, resources);
            ActionSystem::spin(world, resources, cluster.as_mut());
            if Self::strickers_death_check(left, right, world) {
                if let Some(tape) = tape {
                    tape.push(cluster.unwrap());
                }
                return true;
            }

            Event::BeforeStrike {
                owner: left,
                target: right,
            }
            .send(world, resources);
            ActionSystem::spin(world, resources, cluster.as_mut());
            if Self::strickers_death_check(left, right, world) {
                if let Some(tape) = tape {
                    tape.push(cluster.unwrap());
                }
                return true;
            }

            Event::BeforeStrike {
                owner: right,
                target: left,
            }
            .send(world, resources);
            ActionSystem::spin(world, resources, cluster.as_mut());
            if Self::strickers_death_check(left, right, world) {
                if let Some(tape) = tape {
                    tape.push(cluster.unwrap());
                }
                return true;
            }
            if let Some(tape) = tape {
                tape.push({
                    let mut cluster = cluster.unwrap();
                    // cluster.set_duration(1.0);
                    cluster
                });
                cluster = Some(NodeCluster::default());
            }

            let scale = resources.options.floats.slots_striker_scale;
            let (left_hit_pos, right_hit_pos) = (vec2(-1.0 * scale, 0.0), vec2(1.0 * scale, 0.0));

            if let Some(tape) = tape {
                let mut node = Node::default();
                VfxSystem::translate_animated(
                    left,
                    left_hit_pos,
                    &mut node,
                    world,
                    EasingType::Linear,
                    0.03,
                );
                VfxSystem::translate_animated(
                    right,
                    right_hit_pos,
                    &mut node,
                    world,
                    EasingType::Linear,
                    0.03,
                );
                let mut cluster =
                    NodeCluster::new(node.lock(NodeLockType::Full { world, resources }));
                cluster.push(Node::default().lock(NodeLockType::Full { world, resources }));
                cluster.set_duration(0.5);
                tape.push(cluster);
            }

            let duration = 1.5;
            if let Some(cluster) = &mut cluster {
                let mut node = Node::default();
                VfxSystem::translate_animated(
                    left,
                    SlotSystem::get_position(1, &Faction::Light, resources),
                    &mut node,
                    world,
                    EasingType::QuartOut,
                    duration,
                );
                VfxSystem::translate_animated(
                    right,
                    SlotSystem::get_position(1, &Faction::Dark, resources),
                    &mut node,
                    world,
                    EasingType::QuartOut,
                    duration,
                );
                Self::add_strike_vfx(world, resources, &mut node);
                cluster.push(node.lock(NodeLockType::Full { world, resources }));
            }
            Self::hit(left, right, cluster.as_mut(), world, resources);
            ActionSystem::spin(world, resources, cluster.as_mut());
            Event::TurnEnd.send(world, resources);
            ActionSystem::spin(world, resources, cluster.as_mut());
            if let Some(tape) = tape {
                let mut cluster = cluster.unwrap();
                cluster.set_duration(duration);
                tape.push(cluster);
            }
            return true;
        }
        false
    }

    pub fn find_hitters(world: &legion::World) -> Option<(legion::Entity, legion::Entity)> {
        let mut light = None;
        let mut dark = None;
        for (entity, state) in <(&EntityComponent, &ContextState)>::query()
            .filter(component::<UnitComponent>())
            .iter(world)
        {
            if state.get_int(&VarName::Slot, world) == 1 {
                if state.get_faction(&VarName::Faction, world) == Faction::Light {
                    light = Some(entity.entity)
                } else if state.get_faction(&VarName::Faction, world) == Faction::Dark {
                    dark = Some(entity.entity)
                }
            }
        }
        if light.is_some() && dark.is_some() {
            Some((light.unwrap(), dark.unwrap()))
        } else {
            None
        }
    }

    pub fn hit(
        left: legion::Entity,
        right: legion::Entity,
        mut cluster: Option<&mut NodeCluster>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        ActionSystem::spin(world, resources, cluster.as_deref_mut());

        if UnitSystem::is_alive(left, world, resources)
            && UnitSystem::is_alive(right, world, resources)
        {
            resources.action_queue.push_back(Action::new(
                Context::new(ContextLayer::Unit { entity: left }, world, resources)
                    .set_target(right),
                Effect::Damage {
                    value: None,
                    on_hit: None,
                    source: default(),
                }
                .wrap(),
            ));
            resources.action_queue.push_back(Action::new(
                Context::new(ContextLayer::Unit { entity: right }, world, resources)
                    .set_target(left),
                Effect::Damage {
                    value: None,
                    on_hit: None,
                    source: default(),
                }
                .wrap(),
            ));
            ActionSystem::spin(world, resources, cluster.as_deref_mut());
        }

        if UnitSystem::is_alive(left, world, resources) {
            Event::AfterStrike {
                owner: left,
                target: right,
            }
            .send(world, resources);
            ActionSystem::spin(world, resources, cluster.as_deref_mut());
        }

        if UnitSystem::is_alive(right, world, resources) {
            Event::AfterStrike {
                owner: right,
                target: left,
            }
            .send(world, resources);
            ActionSystem::spin(world, resources, cluster.as_deref_mut());
        }
    }

    fn promote_all(world: &mut legion::World) {
        for unit in UnitSystem::collect_factions(world, &Faction::battle()) {
            let state = ContextState::get_mut(unit, world);
            let rank = state.vars.get_int(&VarName::Rank);
            if rank < 3 {
                state.vars.change_int(&VarName::Rank, 1);
            }
        }
    }

    fn add_strike_vfx(_: &mut legion::World, resources: &mut Resources, node: &mut Node) {
        let position = BATTLEFIELD_POSITION;
        node.add_effect(VfxSystem::vfx_strike(resources, position));
    }
}

impl System for BattleSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
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
