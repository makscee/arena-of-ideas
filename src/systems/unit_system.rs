use super::*;

pub struct UnitSystem {}

const CARD_ANIMATION_TIME: Time = 0.2;
impl UnitSystem {
    pub fn pack_shader(shader: &mut Shader, options: &Options) {
        let chain_shader = mem::replace(shader, options.shaders.unit.clone());
        shader.chain_before.push(chain_shader);
    }

    pub fn draw_all_units_to_cassette_node(
        world: &legion::World,
        options: &Options,
        statuses: &StatusPool,
        node: &mut CassetteNode,
        factions: HashSet<Faction>,
    ) {
        <(&UnitComponent, &EntityComponent, &Shader)>::query()
            .iter(world)
            .filter(|(unit, _, _)| factions.contains(&unit.faction))
            .for_each(|(_, entity, _)| {
                let mut unit_shader = ShaderSystem::get_entity_shader(world, entity.entity, None);
                Self::pack_shader(&mut unit_shader, options);
                let context = ContextSystem::get_context(entity.entity, world);
                unit_shader
                    .parameters
                    .uniforms
                    .merge_mut(&context.vars.clone().into(), true);
                unit_shader
                    .chain_before
                    .extend(statuses.get_entity_shaders(&entity.entity));
                unit_shader
                    .chain_after
                    .extend(StatsUiSystem::get_entity_shaders(&context.vars, options));
                unit_shader.chain_after.push(NameSystem::get_entity_shader(
                    entity.entity,
                    world,
                    options,
                ));
                node.add_entity_shader(entity.entity, unit_shader);
            });
        node.save_active_statuses(statuses);
    }

    pub fn process_death(
        entity: legion::Entity,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> bool {
        Event::BeforeDeath { owner: entity }.send(world, resources);
        ActionSystem::run_ticks(world, resources);
        ContextSystem::refresh_entity(entity, world, resources);
        let context = ContextSystem::get_context(entity, world);
        if context.vars.get_int(&VarName::HpValue) <= context.vars.get_int(&VarName::HpDamage) {
            if Self::turn_unit_into_corpse(entity, world, resources) {
                Event::AfterDeath { context }.send(world, resources);
                resources.status_pool.clear_entity(&entity);
                return true;
            }
        }
        false
    }

    pub fn turn_unit_into_corpse(
        entity: legion::Entity,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> bool {
        let unit = world
            .entry_ref(entity)
            .unwrap()
            .get_component::<UnitComponent>()
            .unwrap()
            .clone();
        if unit.faction == Faction::Team {
            Event::RemoveFromTeam { owner: entity }.send(world, resources);
        }
        let corpse = PackedUnit::pack(entity, world, resources);
        resources
            .unit_corpses
            .insert(entity, (corpse, unit.faction));
        let res = world.remove(entity);
        res
    }

    pub fn clear_faction(world: &mut legion::World, resources: &mut Resources, faction: Faction) {
        Self::clear_factions(world, resources, &hashset! {faction});
    }

    pub fn clear_factions(
        world: &mut legion::World,
        resources: &mut Resources,
        factions: &HashSet<Faction>,
    ) -> Vec<legion::Entity> {
        let unit_entitites = <(&EntityComponent, &UnitComponent)>::query()
            .iter(world)
            .filter_map(|(entity, unit)| match factions.contains(&unit.faction) {
                true => Some(entity.entity.clone()),
                false => None,
            })
            .collect_vec();
        unit_entitites.iter().for_each(|entity| {
            world.remove(*entity);
            resources.status_pool.clear_entity(entity);
        });
        unit_entitites
    }

    pub fn collect_faction(
        world: &legion::World,
        faction: Faction,
    ) -> HashMap<legion::Entity, UnitComponent> {
        Self::collect_factions(world, &hashset! {faction})
    }

    pub fn collect_factions(
        world: &legion::World,
        factions: &HashSet<Faction>,
    ) -> HashMap<legion::Entity, UnitComponent> {
        HashMap::from_iter(
            <(&UnitComponent, &EntityComponent)>::query()
                .iter(world)
                .filter_map(|(unit, entity)| match factions.contains(&unit.faction) {
                    true => Some((entity.entity, unit.clone())),
                    false => None,
                }),
        )
    }

    pub fn inject_entity_shaders_uniforms(
        entity_shaders: &mut HashMap<legion::Entity, Shader>,
        resources: &Resources,
    ) {
        for (entity, shader) in entity_shaders.iter_mut() {
            let faction_value = shader
                .parameters
                .uniforms
                .get_float(&VarName::Faction.convert_to_uniform());
            if let Some(faction_value) = faction_value {
                let mut card_value: f32 = match faction_value == Faction::Shop.float_value() {
                    true => 1.0,
                    false => 0.0,
                };
                let hover_value = resources
                    .input
                    .hover_data
                    .get(entity)
                    .unwrap_or(&(false, -1000.0));
                let hover_value = (1.0
                    - hover_value.0 as u8 as f32
                    - ((resources.global_time - hover_value.1) / CARD_ANIMATION_TIME).min(1.0))
                .abs();
                card_value = card_value.max(hover_value);

                shader.parameters.uniforms.insert(
                    VarName::Card.convert_to_uniform(),
                    ShaderUniform::Float(card_value),
                );
                shader.parameters.uniforms.insert(
                    VarName::Zoom.convert_to_uniform(),
                    ShaderUniform::Float(1.0 + hover_value * 1.4),
                );
            }
        }
    }
}
