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
        attacker: legion::Entity,
        victim: legion::Entity,
        amount: usize,
        world: &mut legion::World,
    ) {
        if amount == 0 {
            return;
        }
        let state = ContextState::get_mut(victim, world);
        state.vars.set_entity(&VarName::LastAttacker, attacker);
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

    pub fn pack_shader(shader: &mut Shader, options: &Options) {
        let chain_shader = mem::replace(shader, options.shaders.unit.clone());
        shader.ts = chain_shader.ts;
        shader.chain_before.push(chain_shader);
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
            shader.input_handlers.push(Self::unit_input_handler);
        }
        shader
            .parameters
            .uniforms
            .insert_float_ref(
                &VarName::Card.uniform(),
                match faction {
                    Faction::Shop => 1.0,
                    _ => 0.0,
                },
            )
            .insert_color_ref(
                &VarName::FactionColor.uniform(),
                *options.colors.factions.get(&faction).unwrap(),
            )
            .insert_float_ref(
                &VarName::Scale.uniform(),
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
            .parameters
            .uniforms
            .insert_string_ref(&VarName::HpStr.uniform(), (hp - damage).to_string(), 1)
            .insert_string_ref(&VarName::AttackStr.uniform(), atk.to_string(), 1);
        if damage > 0 {
            shader.set_color_ref("u_hp_color", resources.options.colors.damage);
        } else if hp > original_hp {
            shader.set_color_ref("u_hp_color", resources.options.colors.addition);
        }

        if original_atk < atk {
            shader.set_color_ref("u_attack_color", resources.options.colors.addition);
        }

        shader.entity = Some(entity);

        let status_shaders = StatusLibrary::get_context_shaders(&context, world, resources);
        status_shaders
            .into_iter()
            .for_each(|x| shader.chain_after.insert(0, x));
        node.add_entity_shader(entity, shader);
        node.save_entity_statuses(&entity, &context, world);
        let definitions = UnitSystem::extract_definition_names(entity, world, resources);
        node.save_entity_definitions(entity, definitions);
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
        cluster: &mut Option<NodeCluster>,
    ) -> bool {
        Event::BeforeDeath { owner: entity }.send(world, resources);
        ActionSystem::run_ticks(world, resources, cluster);
        if !UnitSystem::is_alive(entity, world, resources) {
            Self::turn_unit_into_corpse(entity, world, resources);
            return true;
        }
        false
    }

    pub fn turn_unit_into_corpse(
        entity: legion::Entity,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
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
            Event::AfterKill {
                owner: killer,
                target: entity,
            }
            .send(world, resources);
        }
        if faction == Faction::Team {
            Event::RemoveFromTeam { owner: entity }.send(world, resources);
        }
    }

    pub fn revive_corpse(
        entity: legion::Entity,
        slot: Option<usize>,
        world: &mut legion::World,
        logger: &Logger,
    ) {
        let mut entry = world.entry(entity).unwrap();
        entry.remove_component::<CorpseComponent>();
        entry.add_component(UnitComponent {});
        ContextState::get_mut(entity, world)
            .vars
            .set_int(&VarName::HpDamage, 0);
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
            if let Some(team) = TeamSystem::entity(faction, world) {
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
        entity_shaders: &mut HashMap<legion::Entity, Shader>,
        resources: &Resources,
    ) {
        for (entity, shader) in entity_shaders.iter_mut() {
            if let Some(mut card_value) = shader
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

                shader
                    .parameters
                    .uniforms
                    .insert_ref(&VarName::Card.uniform(), ShaderUniform::Float(card_value));
                shader.parameters.uniforms.insert_ref(
                    &VarName::Zoom.uniform(),
                    ShaderUniform::Float(1.0 + hover_value * 1.4),
                );
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

    pub fn extract_definition_names(
        entity: legion::Entity,
        world: &legion::World,
        resources: &Resources,
    ) -> HashSet<String> {
        if let Some(description) = ContextState::get(entity, world)
            .vars
            .try_get_string(&VarName::Description)
        {
            let mut definitions: HashSet<String> = default();
            for definition in resources.definitions_regex.captures_iter(&description) {
                let definition = definition.index(0);
                if resources.definitions.contains(definition) {
                    definitions.insert(definition.to_string());
                }
            }
            return definitions;
        }
        return default();
    }
}
