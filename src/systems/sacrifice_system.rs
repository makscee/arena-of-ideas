use super::*;

pub struct SacrificeSystem;

impl SacrificeSystem {
    pub fn enter_state(world: &mut legion::World, resources: &mut Resources) {
        let vars = &mut TeamSystem::get_state_mut(Faction::Team, world).vars;
        let promotions = vars.try_get_int(&VarName::Promotions).unwrap_or(1) as usize;
        resources.camera.focus = Focus::Shop;
        let phase = SacrificePhase::RankUp { count: promotions };
        phase.init(world, resources);
        resources.sacrifice_data.phase = phase;

        fn input_handler(
            event: HandleEvent,
            _: legion::Entity,
            _: &mut Shader,
            world: &mut legion::World,
            resources: &mut Resources,
        ) {
            match event {
                HandleEvent::Click => {
                    if resources.sacrifice_data.phase.can_accept() {
                        mem::take(&mut resources.sacrifice_data.phase).accept(world, resources);
                    }
                }
                _ => {}
            };
        }
        fn update_handler(
            _: HandleEvent,
            _: legion::Entity,
            shader: &mut Shader,
            _: &mut legion::World,
            resources: &mut Resources,
        ) {
            shader.set_enabled(resources.sacrifice_data.phase.can_accept());
        }
        let entity = new_entity();
        Widget::Button {
            text: "Accept".to_owned(),
            input_handler,
            update_handler: None,
            pre_update_handler: Some(update_handler),
            options: &resources.options,
            uniforms: resources.options.uniforms.ui_button.clone(),
            shader: None,
            entity,
            hover_hints: vec![],
        }
        .generate_node()
        .lock(NodeLockType::Empty)
        .push_as_panel(entity, resources);
    }
}

#[derive(Debug)]
pub enum SacrificePhase {
    RankUp { count: usize },
    Sacrifice { candidates: HashSet<legion::Entity> },
    None,
}

impl SacrificePhase {
    pub fn init(&self, world: &mut legion::World, resources: &mut Resources) {
        match self {
            SacrificePhase::RankUp { .. } => {
                SlotSystem::add_slots_buttons(
                    Faction::Team,
                    "Promote",
                    Some("u_filled"),
                    None,
                    None,
                    world,
                    resources,
                );
            }
            SacrificePhase::Sacrifice { .. } => {
                SlotSystem::add_slots_buttons(
                    Faction::Team,
                    "Sacrifice",
                    Some("u_filled"),
                    None,
                    Some(resources.options.colors.sacrifice),
                    world,
                    resources,
                );
                PanelsSystem::add_text_alert(
                    resources.options.colors.sacrifice,
                    "Sacrifice",
                    "Sacrifice at least 1 hero, get bonuses",
                    vec2::ZERO,
                    vec![PanelFooterButton::Close],
                    resources,
                );
            }
            SacrificePhase::None => {
                SlotSystem::clear_slots_buttons(Faction::Team, world);
            }
        }
    }

    pub fn select_slot(
        mut self,
        slot: usize,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> Self {
        match &mut self {
            SacrificePhase::RankUp { count } => {
                *count -= 1;
                let unit = SlotSystem::find_unit_by_slot(slot, &Faction::Team, world)
                    .expect("Tried to rank up empty slot");
                ContextState::get_mut(unit, world)
                    .vars
                    .change_int(&VarName::Rank, 1);
                if *count == 0 {
                    if UnitSystem::collect_faction(world, Faction::Team).len() == MAX_SLOTS {
                        self = SacrificePhase::Sacrifice {
                            candidates: default(),
                        };
                    } else {
                        self = SacrificePhase::None;
                    }
                    self.init(world, resources);
                }
            }
            SacrificePhase::Sacrifice { candidates } => {
                let unit = SlotSystem::find_unit_by_slot(slot, &Faction::Team, world)
                    .expect("Tried to mark empty slot");
                if candidates.contains(&unit) {
                    candidates.remove(&unit);
                    resources
                        .tape_player
                        .tape
                        .close_panels(unit, resources.tape_player.head);
                } else {
                    candidates.insert(unit);
                    let position = SlotSystem::get_position(slot, &Faction::Team, resources);
                    let text = format!(
                        "+{} g",
                        ContextState::get(unit, world).get_int(&VarName::Rank, world) + 1
                    );
                    Node::new_panel_scaled(
                        resources
                            .options
                            .shaders
                            .slot_sacrifice_marker
                            .clone()
                            .insert_vec2("u_position".to_owned(), position)
                            .insert_color("u_color".to_owned(), resources.options.colors.subtract)
                            .insert_string("u_g_text".to_owned(), text, 1),
                    )
                    .lock(NodeLockType::Empty)
                    .push_as_panel(unit, resources);
                }
            }
            SacrificePhase::None => panic!(),
        }
        self
    }

    pub fn accept(&self, world: &mut legion::World, resources: &mut Resources) {
        match self {
            SacrificePhase::None => {}
            SacrificePhase::Sacrifice { candidates } => {
                debug!("Sacrifice {candidates:?}");
                let mut sum = 0;
                for unit in candidates.iter() {
                    let context =
                        Context::new(ContextLayer::Unit { entity: *unit }, world, resources)
                            .set_target(*unit);
                    sum += context.get_int(&VarName::Rank, world).unwrap() + 1;
                    Effect::Kill.wrap().push(context, resources);
                    ActionSystem::spin(world, resources, None);
                    ActionSystem::death_check(world, resources, None);
                }
                ShopSystem::change_g(sum, Some("Sacrifice"), world, resources);
            }
            SacrificePhase::RankUp { .. } => {
                panic!()
            }
        }
        GameStateSystem::set_transition(GameState::Shop, resources);
    }

    pub fn can_accept(&self) -> bool {
        match self {
            SacrificePhase::RankUp { .. } => false,
            SacrificePhase::Sacrifice { candidates } => !candidates.is_empty(),
            SacrificePhase::None => true,
        }
    }
}

impl Default for SacrificePhase {
    fn default() -> Self {
        Self::None
    }
}
