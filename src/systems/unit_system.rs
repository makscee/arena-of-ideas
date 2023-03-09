use super::*;

pub struct UnitSystem {}

const STATUSES_EFFECTS_KEY: &str = "statuses";
const CARD_ANIMATION_DURATION: Time = 0.2;
impl UnitSystem {
    pub fn draw_all_units_to_cassette_node(
        world: &legion::World,
        options: &Options,
        statuses: &StatusPool,
        houses: &HashMap<HouseName, House>,
        node: &mut CassetteNode,
        factions: HashSet<Faction>,
    ) {
        node.clear_key(STATUSES_EFFECTS_KEY);
        <(&UnitComponent, &EntityComponent, &Shader)>::query()
            .iter(world)
            .filter(|(unit, _, _)| factions.contains(&unit.faction))
            .for_each(|(_, entity, _)| {
                let mut unit_shader = ShaderSystem::get_entity_shader(world, entity.entity).clone();
                unit_shader.chain = Some(Box::new(
                    [
                        statuses.get_entity_shaders(&entity.entity),
                        match unit_shader.chain {
                            Some(chain) => chain.deref().clone(),
                            None => vec![],
                        },
                    ]
                    .concat(),
                ));
                node.add_entity_shader(entity.entity, unit_shader);
                statuses
                    .get_description_shaders(&entity.entity, options)
                    .into_iter()
                    .for_each(|shader| {
                        node.add_effect_by_key(
                            STATUSES_EFFECTS_KEY,
                            VisualEffect {
                                duration: 0.0,
                                delay: 0.0,
                                r#type: VisualEffectType::EntityExtraShaderConst {
                                    entity: entity.entity,
                                    shader,
                                },
                                order: 30,
                            },
                        )
                    });
            });
        StatsUiSystem::fill_cassette_node(world, options, node);
        HouseSystem::fill_cassette_node_with_descriptions(world, options, houses, node);
        NameSystem::fill_cassette_node(world, options, node);
    }

    pub fn duplicate_unit(
        original_entity: legion::Entity,
        world: &mut legion::World,
        resources: &mut Resources,
        faction: Faction,
    ) {
        let original_entry = world.entry_ref(original_entity).unwrap();
        let original_context = original_entry.get_component::<Context>().unwrap().clone();
        // Mandatory components
        // For some reasoun it seems to be possible to init entity only with 8 components
        // Any more mandatory components should be cloned down below
        let new_entity = world.push((
            {
                let mut unit = original_entry
                    .get_component::<UnitComponent>()
                    .unwrap()
                    .clone();
                unit.faction = faction;
                unit
            },
            original_entry
                .get_component::<PositionComponent>()
                .unwrap()
                .clone(),
            original_entry
                .get_component::<RadiusComponent>()
                .unwrap()
                .clone(),
            original_entry
                .get_component::<HpComponent>()
                .unwrap()
                .clone(),
            original_entry
                .get_component::<AttackComponent>()
                .unwrap()
                .clone(),
            original_entry
                .get_component::<NameComponent>()
                .unwrap()
                .clone(),
            original_entry
                .get_component::<FlagsComponent>()
                .unwrap()
                .clone(),
            original_entry
                .get_component::<HouseComponent>()
                .unwrap()
                .clone(),
        ));
        let parent = WorldSystem::get_context(world).owner;
        let mut new_entry = world.entry(new_entity).unwrap();
        new_entry.add_component(EntityComponent { entity: new_entity });
        new_entry.add_component(Context {
            owner: new_entity,
            parent: Some(parent),
            target: new_entity,
            ..original_context
        });

        // Optional components
        Self::clone_component::<Shader>(original_entity, new_entity, world);
        Self::clone_component::<DescriptionComponent>(original_entity, new_entity, world);

        // // Statuses
        if let Some(statuses) = resources.status_pool.active_statuses.get(&original_entity) {
            statuses.clone().iter().for_each(|(name, context)| {
                StatusPool::add_entity_status(
                    new_entity,
                    name,
                    Context {
                        owner: new_entity,
                        target: new_entity,
                        parent: Some(new_entity),
                        ..context.clone()
                    },
                    resources,
                );
            });
        }
    }

    fn clone_component<T: legion::storage::Component>(
        original_entity: legion::Entity,
        clone_entity: legion::Entity,
        world: &mut legion::World,
    ) where
        T: Clone,
    {
        if let Some(component) = world
            .entry_ref(original_entity)
            .unwrap()
            .get_component::<T>()
            .cloned()
            .ok()
        {
            world.entry(clone_entity).unwrap().add_component(component);
        }
    }

    pub fn add_attention_vars(unit: &UnitComponent, entry: &legion::world::Entry, vars: &mut Vars) {
        let card = match unit.faction == Faction::Shop {
            true => 1.0,
            false => {
                let attention_ts = entry
                    .get_component::<AttentionComponent>()
                    .map_or(0.0, |component| component.ts);
                (attention_ts / CARD_ANIMATION_DURATION).clamp(0.0, 1.0)
            }
        };

        vars.insert(VarName::Card, Var::Float(card));
    }

    pub fn kill(
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
            Event::RemoveFromTeam {
                context: ContextSystem::get_context(entity, world),
            }
            .send(resources, world);
        }
        resources.status_pool.clear_entity(&entity);
        let res = world.remove(entity);
        res
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
}
