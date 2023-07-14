use super::*;

pub struct UnitSystem {}

const CARD_ANIMATION_TIME: Time = 0.2;
impl UnitSystem {
    pub fn unit_string(
        entity: legion::Entity,
        world: &legion::World,
        resources: &Resources,
    ) -> String {
        let state = Context::new(ContextLayer::Unit { entity }, world, resources);
        format!(
            "{} {}/{}",
            state.name(world).unwrap(),
            state.get_int(&VarName::AttackValue, world).unwrap(),
            state.get_int(&VarName::HpValue, world).unwrap(),
        )
    }

    pub fn is_alive(entity: legion::Entity, world: &legion::World, resources: &Resources) -> bool {
        let context = Context::new(ContextLayer::Unit { entity }, world, resources);
        context.get_int(&VarName::HpValue, world) > context.get_int(&VarName::HpDamage, world)
    }

    pub fn all(world: &legion::World) -> Vec<legion::Entity> {
        <&EntityComponent>::query()
            .filter(component::<UnitComponent>())
            .iter(world)
            .map(|x| x.entity)
            .collect_vec()
    }

    pub fn deal_damage(
        caster: legion::Entity,
        victim: legion::Entity,
        amount: usize,
        world: &mut legion::World,
    ) {
        if amount == 0 {
            return;
        }
        let state = ContextState::get_mut(victim, world);
        state.vars.set_entity(&VarName::LastAttacker, caster);
        let damage = (state
            .vars
            .try_get_int(&VarName::HpDamage)
            .unwrap_or_default() as usize
            + amount)
            .min(i32::MAX as usize) as i32;
        state.vars.set_int(&VarName::HpDamage, damage);
    }

    pub fn heal_damage(
        healer: legion::Entity,
        victim: legion::Entity,
        amount: usize,
        world: &mut legion::World,
    ) {
        if amount == 0 {
            return;
        }

        let state = ContextState::get_mut(victim, world);
        state.vars.set_entity(&VarName::LastHealer, healer);
        let amount = (amount as i32).min(state.vars.get_int(&VarName::HpDamage));
        state.vars.change_int(&VarName::HpDamage, -amount);
    }

    pub fn draw_unit_to_node(
        entity: legion::Entity,
        node: &mut Node,
        world: &legion::World,
        resources: &Resources,
    ) {
        let options = &resources.options;
        let context = Context::new(ContextLayer::Unit { entity }, world, resources);
        let mut shader = ShaderSystem::get_entity_shader(entity, world)
            .unwrap()
            .add_context(&context, world);
        let faction = context.get_faction(&VarName::Faction, world).unwrap();
        let slot = context.get_int(&VarName::Slot, world).unwrap() as usize;
        if faction == Faction::Shop || faction == Faction::Team {
            shader.middle.input_handlers.push(Self::unit_input_handler);
        }
        shader
            .insert_float_ref(
                "u_card".to_owned(),
                match faction {
                    Faction::Shop => 1.0,
                    _ => 0.0,
                },
            )
            .insert_color_ref("u_faction_color".to_owned(), faction.color(options))
            .insert_float_ref(
                "u_scale".to_owned(),
                SlotSystem::get_scale(slot, faction, resources),
            );
        let original_hp = context.get_int(&VarName::HpOriginalValue, world).unwrap();
        let original_atk = context
            .get_int(&VarName::AttackOriginalValue, world)
            .unwrap();
        let hp = context.get_int(&VarName::HpValue, world).unwrap();
        let damage = context
            .get_int(&VarName::HpDamage, world)
            .unwrap_or_default();
        let atk = context.get_int(&VarName::AttackValue, world).unwrap();
        shader
            .insert_string_ref("u_hp_str".to_owned(), (hp - damage).to_string(), 1)
            .insert_string_ref("u_attack_str".to_owned(), atk.to_string(), 1);
        let rank = context.get_int(&VarName::Rank, world).unwrap();
        if rank > 0 {
            shader
                .insert_float_ref("u_rank_1".to_owned(), 1.0)
                .insert_float_ref("u_rank_2".to_owned(), (rank > 1) as i32 as f32)
                .insert_float_ref("u_rank_3".to_owned(), (rank > 2) as i32 as f32);

            shader.middle.hover_hints.push((
                resources.options.colors.primary,
                format!("Rank {rank}"),
                format!("+{rank}/+{rank}"),
            ));
        }
        shader.insert_color_ref(
            "u_hp_color".to_owned(),
            if damage > 0 {
                resources.options.colors.damage
            } else if hp > original_hp {
                resources.options.colors.add
            } else {
                resources.options.colors.text
            },
        );

        shader.insert_color_ref(
            "u_attack_color".to_owned(),
            if original_atk < atk {
                resources.options.colors.add
            } else {
                resources.options.colors.text
            },
        );

        shader.middle.entity = Some(entity);

        let status_shaders = StatusLibrary::get_context_shaders(&context, world, resources);
        status_shaders
            .into_iter()
            .for_each(|x| shader.after.insert(0, x));
        let statuses = context.collect_statuses(world);
        StatusSystem::add_active_statuses_hint(&mut shader.middle, &statuses, resources);
        let mut definitions = UnitSystem::extract_definitions_from_unit(entity, world, resources);
        definitions.extend(statuses.into_iter().map(|(name, _)| name));
        Definitions::add_hints(&mut shader.middle, definitions, resources);
        node.add_entity_shader(entity, shader);
    }

    pub fn draw_all_units_to_node(
        factions: &HashSet<Faction>,
        node: &mut Node,
        world: &legion::World,
        resources: &Resources,
    ) {
        let units = UnitSystem::collect_factions(world, factions);
        for entity in units.iter() {
            Self::draw_unit_to_node(*entity, node, world, resources)
        }
    }

    pub fn process_death(
        entity: legion::Entity,
        world: &mut legion::World,
        resources: &mut Resources,
        cluster: Option<&mut NodeCluster>,
    ) -> Option<legion::Entity> {
        Event::BeforeDeath { owner: entity }.send(world, resources);
        ActionSystem::run_ticks(world, resources, cluster);
        if !UnitSystem::is_alive(entity, world, resources) {
            return Some(Self::turn_unit_into_corpse(entity, world, resources));
        }
        None
    }

    pub fn turn_unit_into_corpse(
        entity: legion::Entity,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> legion::Entity {
        let state = ContextState::get(entity, world);
        let killer = state.vars.try_get_entity(&VarName::LastAttacker);
        let faction = state.get_faction(&VarName::Faction, world);
        let corpse: CorpseComponent = CorpseComponent::from_unit(killer.unwrap_or(entity));

        let mut entry = world.entry(entity).unwrap();
        entry.remove_component::<UnitComponent>();
        entry.add_component(corpse);

        resources.logger.log(
            || format!("{:?} is now corpse", entity),
            &LogContext::UnitCreation,
        );
        Event::AfterDeath { owner: entity }.send(world, resources);
        ContextState::get_mut(entity, world).statuses.clear();
        if let Some(killer) = killer {
            if killer != entity {
                Event::AfterKill {
                    owner: killer,
                    target: entity,
                }
                .send(world, resources);
            }
        }
        if faction == Faction::Team {
            //todo: bug on rebirth?
            Event::RemoveFromTeam { owner: entity }.send(world, resources);
        }
        killer.unwrap_or(entity)
    }

    pub fn revive_corpse(entity: legion::Entity, world: &mut legion::World, logger: &Logger) {
        let mut entry = world.entry(entity).unwrap();
        entry.remove_component::<CorpseComponent>();
        entry.add_component(UnitComponent {});
        let vars = &mut ContextState::get_mut(entity, world).vars;
        vars.set_int(&VarName::HpDamage, 0);
        logger.log(
            || format!("Corpse revived {entity:?}"),
            &LogContext::UnitCreation,
        );
    }

    pub fn clear_faction(world: &mut legion::World, resources: &mut Resources, faction: Faction) {
        Self::clear_factions(world, &hashset! {faction});
    }

    pub fn clear_factions(
        world: &mut legion::World,
        factions: &HashSet<Faction>,
    ) -> Vec<legion::Entity> {
        let entities = Self::collect_entities(factions, world);
        for entity in entities.iter() {
            world.remove(*entity);
        }
        for faction in factions {
            if let Some(team) = TeamSystem::entity(*faction, world) {
                world.remove(team);
            }
        }
        entities
    }

    pub fn collect_entities(
        factions: &HashSet<Faction>,
        world: &legion::World,
    ) -> Vec<legion::Entity> {
        <(&EntityComponent, &ContextState)>::query()
            .filter(component::<UnitComponent>() | component::<CorpseComponent>())
            .iter(world)
            .filter_map(|(entity, state)| {
                if factions.contains(&state.get_faction(&VarName::Faction, world)) {
                    Some(entity.entity)
                } else {
                    None
                }
            })
            .collect_vec()
    }

    pub fn collect_faction(world: &legion::World, faction: Faction) -> Vec<legion::Entity> {
        Self::collect_factions(world, &hashset! {faction})
    }

    pub fn collect_factions(
        world: &legion::World,
        factions: &HashSet<Faction>,
    ) -> Vec<legion::Entity> {
        Self::collect_factions_states(world, factions)
            .into_iter()
            .map(|(entity, _)| entity)
            .collect_vec()
    }

    pub fn collect_faction_states<'a>(
        world: &'a legion::World,
        faction: Faction,
    ) -> HashMap<legion::Entity, &'a ContextState> {
        Self::collect_factions_states(world, &hashset! {faction})
    }

    pub fn collect_factions_states<'a>(
        world: &'a legion::World,
        factions: &HashSet<Faction>,
    ) -> HashMap<legion::Entity, &'a ContextState> {
        HashMap::from_iter(
            <(&EntityComponent, &ContextState)>::query()
                .filter(component::<UnitComponent>())
                .iter(world)
                .filter_map(|(entity, state)| {
                    match factions.contains(&state.get_faction(&VarName::Faction, world)) {
                        true => Some((entity.entity, state)),
                        false => None,
                    }
                }),
        )
    }

    pub fn get_corpse(entity: legion::Entity, world: &legion::World) -> Option<CorpseComponent> {
        world
            .entry_ref(entity)
            .ok()
            .and_then(|x| x.get_component::<CorpseComponent>().ok().cloned())
    }

    pub fn inject_entity_shaders_uniforms(
        entity_shaders: &mut HashMap<legion::Entity, ShaderChain>,
        resources: &Resources,
    ) {
        if resources.current_state == GameState::Battle
            && resources.tape_player.mode == TapePlayMode::Play
        {
            return;
        }
        for (entity, shader) in entity_shaders.iter_mut() {
            if let Some(mut card_value) = shader
                .middle
                .parameters
                .uniforms
                .try_get_float(&VarName::Card.uniform())
            {
                let hover_value = resources
                    .input_data
                    .input_events
                    .get(entity)
                    .and_then(|x| match x.0 {
                        HandleEvent::HoverStart => Some((true, x.1)),
                        HandleEvent::HoverStop => Some((false, x.1 + 0.0)),
                        _ => None,
                    })
                    .unwrap_or((false, -1000.0));
                let hover_value = (1.0
                    - hover_value.0 as u8 as f32
                    - ((resources.global_time - hover_value.1) / CARD_ANIMATION_TIME)
                        .clamp(0.0, 1.0))
                .abs();
                let dragged_value =
                    (resources.input_data.dragged_entity == Some(*entity)) as i32 as f32;
                card_value = (card_value.max(hover_value) - dragged_value).max(0.0);

                shader.insert_float_ref("u_card".to_owned(), card_value);
                shader.insert_float_ref("u_zoom".to_owned(), 1.0 + hover_value * 1.4);
            }
        }
    }

    fn unit_input_handler(
        event: HandleEvent,
        entity: legion::Entity,
        _: &mut Shader,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        if GameStateSystem::current_is(GameState::Shop, resources) {
            match event {
                HandleEvent::DragStop => {
                    SlotSystem::handle_unit_drop(entity, world, resources);
                }
                HandleEvent::Drag { delta } => {
                    ContextState::get_mut(entity, world)
                        .vars
                        .change_vec2(&VarName::Position, delta);
                    SlotSystem::handle_unit_drag(entity, world, resources);
                }

                _ => {}
            };
        }
    }

    pub fn extract_definitions_from_unit(
        entity: legion::Entity,
        world: &legion::World,
        resources: &Resources,
    ) -> HashSet<String> {
        if let Some(description) = ContextState::get(entity, world)
            .vars
            .try_get_string(&VarName::Description)
        {
            return Self::extract_definitions(&description, resources);
        }
        return default();
    }

    pub fn extract_definitions(text: &str, resources: &Resources) -> HashSet<String> {
        let mut definitions: HashSet<String> = default();
        for definition in resources.definitions_regex.captures_iter(text) {
            let definition = definition.index(0);
            if resources.definitions.contains(definition) {
                definitions.insert(definition.to_string());
            }
        }
        return definitions;
    }
}
