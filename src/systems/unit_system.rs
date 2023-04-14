use super::*;
use regex::*;

pub struct UnitSystem {}

const CARD_ANIMATION_TIME: Time = 0.2;
impl UnitSystem {
    pub fn pack_shader(shader: &mut Shader, options: &Options) {
        let chain_shader = mem::replace(shader, options.shaders.unit.clone());
        shader.chain_before.push(chain_shader);
    }

    pub fn get_unit(entity: legion::Entity, world: &legion::World) -> UnitComponent {
        world
            .entry_ref(entity)
            .unwrap()
            .get_component::<UnitComponent>()
            .unwrap()
            .clone()
    }

    pub fn set_slot(entity: legion::Entity, slot: usize, world: &mut legion::World) {
        world
            .entry_mut(entity)
            .unwrap()
            .get_component_mut::<UnitComponent>()
            .unwrap()
            .slot = slot;
    }

    pub fn set_faction(entity: legion::Entity, faction: Faction, world: &mut legion::World) {
        world
            .entry_mut(entity)
            .unwrap()
            .get_component_mut::<UnitComponent>()
            .unwrap()
            .faction = faction;
    }

    pub fn unit_string(entity: legion::Entity, world: &legion::World) -> String {
        let context = ContextSystem::get_context(entity, world);
        let name = world
            .entry_ref(entity)
            .unwrap()
            .get_component::<NameComponent>()
            .unwrap()
            .0
            .clone();
        format!(
            "{name} {}/{}",
            context.vars.get_int(&VarName::HpValue),
            context.vars.get_int(&VarName::AttackValue)
        )
    }

    pub fn draw_unit_to_node(
        entity: legion::Entity,
        unit: &UnitComponent,
        node: &mut Node,
        world: &legion::World,
        resources: &Resources,
    ) {
        let options = &resources.options;
        let mut shader = match ShaderSystem::get_entity_shader(world, entity, None) {
            Some(mut shader) => {
                Self::pack_shader(&mut shader, options);
                shader
            }
            None => options.shaders.unit.clone(),
        };
        if unit.faction == Faction::Shop
            || unit.faction == Faction::Team
            || unit.faction == Faction::Sacrifice
        {
            shader.input_handlers.push(Self::unit_input_handler);
        }
        shader.entity = Some(entity);
        let ts = world
            .entry_ref(entity)
            .unwrap()
            .get_component::<EntityComponent>()
            .unwrap()
            .ts;
        shader.ts = ts;
        let context = ContextSystem::get_context(entity, world);
        shader
            .parameters
            .uniforms
            .merge_mut(&context.vars.clone().into(), true);
        let statuses = &resources.status_pool;
        shader
            .chain_before
            .extend(statuses.get_entity_shaders(&entity));
        shader
            .chain_after
            .extend(StatsUiSystem::get_entity_shaders(&context.vars, options));
        shader
            .chain_after
            .push(NameSystem::get_entity_shader(entity, world, options));
        node.add_entity_shader(entity, shader);
        node.save_entity_statuses(entity, statuses);
        let definitions = UnitSystem::extract_definition_names(entity, world, resources);
        node.save_entity_definitions(entity, definitions);
    }

    pub fn draw_all_units_to_node(
        factions: &HashSet<Faction>,
        node: &mut Node,
        world: &legion::World,
        resources: &Resources,
    ) -> HashMap<legion::Entity, UnitComponent> {
        let units = UnitSystem::collect_factions_units(world, resources, factions, false);
        for (entity, unit) in units.iter() {
            Self::draw_unit_to_node(*entity, unit, node, world, resources)
        }
        units
    }

    pub fn process_death(
        entity: legion::Entity,
        world: &mut legion::World,
        resources: &mut Resources,
        cluster: &mut Option<NodeCluster>,
    ) -> bool {
        Event::BeforeDeath { owner: entity }.send(world, resources);
        ActionSystem::run_ticks(world, resources, cluster);
        let context = ContextSystem::refresh_entity(entity, world, resources);
        if context.vars.get_int(&VarName::HpValue) <= context.vars.get_int(&VarName::HpDamage) {
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
        let mut entry = world.entry(entity).unwrap();
        let unit = entry.get_component::<UnitComponent>().unwrap().clone();
        entry.remove_component::<UnitComponent>();

        let killer = entry
            .get_component::<HealthComponent>()
            .unwrap()
            .last_attacker();
        let faction = unit.faction;
        let corpse: CorpseComponent = CorpseComponent::from_unit(unit, killer.unwrap_or(entity));
        entry.add_component(corpse);
        let corpse_context = ContextSystem::get_context(entity, world);

        resources.logger.log(
            &format!("{:?} is now corpse", entity),
            &LogContext::UnitCreation,
        );
        Event::AfterDeath {
            context: corpse_context,
        }
        .send(world, resources);
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
        StatusPool::clear_entity(&entity, resources);
    }

    pub fn revive_corpse(entity: legion::Entity, slot: Option<usize>, world: &mut legion::World) {
        let mut entry = world.entry(entity).unwrap();
        let corpse = entry.get_component::<CorpseComponent>().unwrap().clone();
        entry.remove_component::<CorpseComponent>();
        let mut unit: UnitComponent = corpse.into();
        unit.slot = slot.unwrap_or_default();
        entry.add_component(unit);
        entry
            .get_component_mut::<HealthComponent>()
            .unwrap()
            .heal_damage(i32::MAX as usize);
    }

    pub fn clear_faction(world: &mut legion::World, resources: &mut Resources, faction: Faction) {
        Self::clear_factions(world, resources, &hashset! {faction});
    }

    pub fn clear_factions(
        world: &mut legion::World,
        resources: &mut Resources,
        factions: &HashSet<Faction>,
    ) -> Vec<legion::Entity> {
        let unit_entitites = Self::collect_entities(factions, world);
        unit_entitites
            .iter()
            .for_each(|entity| Self::delete_unit(*entity, world, resources));
        factions
            .into_iter()
            .for_each(|f| resources.team_states.clear(*f));
        unit_entitites
    }

    pub fn collect_entities(
        factions: &HashSet<Faction>,
        world: &legion::World,
    ) -> Vec<legion::Entity> {
        <(&EntityComponent, &UnitComponent)>::query()
            .iter(world)
            .map(|(entity, unit)| (entity.entity, unit.faction))
            .chain(
                <(&EntityComponent, &CorpseComponent)>::query()
                    .iter(world)
                    .map(|(entity, corpse)| (entity.entity, corpse.faction)),
            )
            .filter_map(|(entity, faction)| match factions.contains(&faction) {
                true => Some(entity.clone()),
                false => None,
            })
            .collect_vec()
    }

    pub fn unit_on_field(unit: &UnitComponent, resources: &Resources) -> bool {
        resources.team_states.get_slots(&unit.faction) >= unit.slot
    }

    pub fn delete_unit(
        entity: legion::Entity,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        world.remove(entity);
        StatusPool::clear_entity(&entity, resources);
    }

    pub fn collect_faction(
        world: &legion::World,
        resources: &Resources,
        faction: Faction,
        force: bool,
    ) -> Vec<legion::Entity> {
        Self::collect_factions(world, resources, &hashset! {faction}, force)
    }

    pub fn collect_factions(
        world: &legion::World,
        resources: &Resources,
        factions: &HashSet<Faction>,
        force: bool,
    ) -> Vec<legion::Entity> {
        Self::collect_factions_units(world, resources, factions, force)
            .into_iter()
            .map(|(entity, _)| entity)
            .collect_vec()
    }

    pub fn collect_faction_units(
        world: &legion::World,
        resources: &Resources,
        faction: Faction,
        force: bool,
    ) -> HashMap<legion::Entity, UnitComponent> {
        Self::collect_factions_units(world, resources, &hashset! {faction}, force)
    }

    pub fn collect_factions_units(
        world: &legion::World,
        resources: &Resources,
        factions: &HashSet<Faction>,
        force: bool,
    ) -> HashMap<legion::Entity, UnitComponent> {
        HashMap::from_iter(
            <(&UnitComponent, &EntityComponent)>::query()
                .iter(world)
                .filter_map(|(unit, entity)| {
                    match factions.contains(&unit.faction)
                        && (force
                            || unit.slot
                                <= resources.team_states.get_team_state(&unit.faction).slots)
                    {
                        true => Some((entity.entity, unit.clone())),
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
            let faction_value = shader
                .parameters
                .uniforms
                .try_get_float(&VarName::Faction.uniform());
            if let Some(faction_value) = faction_value {
                let faction = Faction::from(faction_value);
                let mut card_value: f32 = match faction {
                    Faction::Shop => 1.0,
                    _ => 0.0,
                };
                let hover_value = resources
                    .input_data
                    .input_events
                    .get(entity)
                    .and_then(|x| match x.0 {
                        InputEvent::HoverStart => Some((true, x.1)),
                        InputEvent::HoverStop => Some((false, x.1)),
                        _ => None,
                    })
                    .unwrap_or((false, -1000.0));
                let hover_value = (1.0
                    - hover_value.0 as u8 as f32
                    - ((resources.global_time - hover_value.1) / CARD_ANIMATION_TIME).min(1.0))
                .abs();
                card_value = card_value.max(hover_value);

                shader
                    .parameters
                    .uniforms
                    .insert_ref(&VarName::Card.uniform(), ShaderUniform::Float(card_value));
                shader.parameters.uniforms.insert_ref(
                    &VarName::Zoom.uniform(),
                    ShaderUniform::Float(1.0 + hover_value * 1.4),
                );
                let slot = shader
                    .parameters
                    .uniforms
                    .try_get_int(&VarName::Slot.uniform())
                    .unwrap() as usize;
                shader.parameters.uniforms.insert_ref(
                    &VarName::Scale.uniform(),
                    ShaderUniform::Float(mix(
                        SlotSystem::get_scale(slot, faction, resources),
                        1.0,
                        card_value,
                    )),
                );
            }
        }
    }

    fn unit_input_handler(
        event: InputEvent,
        entity: legion::Entity,
        _: &mut Shader,
        world: &mut legion::World,
        resources: &mut Resources,
    ) {
        match event {
            InputEvent::DragStop => {
                SlotSystem::handle_unit_drop(entity, world, resources);
            }
            InputEvent::Drag { delta } => {
                if let Some(mut entry) = world.entry(entity) {
                    if let Ok(area) = entry.get_component_mut::<AreaComponent>() {
                        area.position += delta;
                        SlotSystem::handle_unit_drag(entity, world, resources);
                    }
                }
            }
            _ => {}
        };
    }

    pub fn extract_definition_names(
        entity: legion::Entity,
        world: &legion::World,
        resources: &Resources,
    ) -> HashSet<String> {
        if let Some(description) = world.entry_ref(entity).ok().and_then(|x| {
            x.get_component::<DescriptionComponent>()
                .ok()
                .and_then(|x| Some(x.text.clone()))
        }) {
            let definitions_regex = Regex::new(r"\b[A-Z][a-zA-Z]*\b").unwrap();
            let mut definitions: HashSet<String> = default();
            for definition in definitions_regex.captures_iter(&description) {
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
