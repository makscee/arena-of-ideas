use super::*;

pub struct SacrificeSystem;

impl SacrificeSystem {
    pub fn enter_state(world: &mut legion::World, resources: &mut Resources) {
        PanelsSystem::close_all_alerts(resources);
        SlotSystem::add_slots_buttons(
            Faction::Team,
            "Sacrifice",
            Some("u_filled"),
            None,
            Some(resources.options.colors.sacrifice),
            world,
            resources,
        );

        fn input_handler(
            event: HandleEvent,
            _: legion::Entity,
            _: &mut Shader,
            world: &mut legion::World,
            resources: &mut Resources,
        ) {
            match event {
                HandleEvent::Click => {
                    let candidates = mem::take(&mut resources.sacrifice_data.candidates);
                    SacrificeSystem::accept_sacrifice(candidates, world, resources)
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
            shader.set_enabled(!resources.sacrifice_data.candidates.is_empty());
        }
        let entity = new_entity();
        Widget::Button {
            text: "Accept".to_owned(),
            color: None,
            input_handler,
            update_handler: None,
            pre_update_handler: Some(update_handler),
            options: &resources.options,
            uniforms: resources.options.uniforms.shop_top_button.clone(),
            shader: None,
            entity,
            hover_hints: vec![],
        }
        .generate_node()
        .lock(NodeLockType::Empty)
        .push_as_panel(entity, resources);
    }

    pub fn handle_slot_activation(
        slot: usize,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        let unit = SlotSystem::find_unit_by_slot(slot, &Faction::Team, world)
            .expect("Tried to mark empty slot");
        let candidates = &mut resources.sacrifice_data.candidates;
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

    fn accept_sacrifice(
        candidates: HashSet<legion::Entity>,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        debug!("Sacrifice {candidates:?}");
        let mut sum = 0;
        for unit in candidates.iter() {
            let context = Context::new(ContextLayer::Unit { entity: *unit }, world, resources)
                .set_target(*unit);
            sum += context.get_int(&VarName::Rank, world).unwrap() + 1;
            Effect::Kill.wrap().push(context, resources);
            ActionSystem::spin(world, resources, None);
            ActionSystem::death_check(world, resources, None);
        }
        ShopSystem::change_g(sum, Some("Sacrifice"), world, resources);
        PanelsSystem::close_all_alerts(resources);
        GameStateSystem::set_transition(GameState::Shop, resources);
    }
}
