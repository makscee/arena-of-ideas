use geng::prelude::itertools::Itertools;
use legion::EntityStore;

use super::*;

#[derive(Debug, Clone)]
pub struct UnitComponent {
    pub slot: usize,
    pub faction: Faction,
}

pub const UNIT_RADIUS: f32 = 1.0;

#[derive(Clone, Copy, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum Faction {
    Light,
    Dark,
    Team,
    Shop,
}

impl Faction {
    pub fn color(&self, options: &Options) -> Rgba<f32> {
        *options.faction_colors.get(self).unwrap()
    }
}

impl VarsProvider for UnitComponent {
    fn extend_vars(&self, vars: &mut Vars) {
        let faction_val = match self.faction {
            Faction::Dark => 0.0,
            Faction::Light => 1.0,
            Faction::Team => 2.0,
            Faction::Shop => 3.0,
        };
        vars.insert(VarName::Faction, Var::Float(faction_val));
        vars.insert(
            VarName::Card,
            Var::Float((self.faction == Faction::Shop) as i32 as f32),
        );
    }
}

const STATUSES_EFFECTS_KEY: &str = "statuses";

impl UnitComponent {
    pub fn new(slot: usize, faction: Faction) -> Self {
        Self { slot, faction }
    }

    pub fn draw_all_units_to_cassette_node(
        world: &legion::World,
        options: &Options,
        statuses: &Statuses,
        node: &mut CassetteNode,
        factions: HashSet<Faction>,
    ) {
        node.clear_key(STATUSES_EFFECTS_KEY);
        <(&UnitComponent, &EntityComponent, &Shader)>::query()
            .iter(world)
            .filter(|(unit, _, _)| factions.contains(&unit.faction))
            .for_each(|(unit, entity, shader)| {
                // Render units
                node.add_entity_shader(
                    entity.entity,
                    ShaderSystem::get_entity_shader(world, entity.entity).clone(),
                );

                // Render statuses
                node.add_effects_by_key(
                    STATUSES_EFFECTS_KEY,
                    statuses
                        .get_entity_shaders(&entity.entity)
                        .into_iter()
                        .map(|shader| {
                            VisualEffect::new(
                                0.0,
                                VisualEffectType::EntityExtraShaderConst {
                                    entity: entity.entity,
                                    shader,
                                },
                                0,
                            )
                        })
                        .collect_vec(),
                )
            });
        StatsUiSystem::fill_cassette_node(world, options, node);
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
        ));

        let mut new_entry = world.entry(new_entity).unwrap();
        new_entry.add_component(EntityComponent { entity: new_entity });
        new_entry.add_component(Context {
            owner: new_entity,
            creator: new_entity,
            target: new_entity,
            ..original_context
        });

        // Optional components
        Self::clone_component::<Shader>(original_entity, new_entity, world);
        Self::clone_component::<DescriptionComponent>(original_entity, new_entity, world);

        // // Statuses
        if let Some(statuses) = resources.statuses.active_statuses.get(&original_entity) {
            statuses.clone().iter().for_each(|(name, context)| {
                resources.statuses.add_entity_status(
                    &new_entity,
                    name,
                    Context {
                        owner: new_entity,
                        target: new_entity,
                        ..context.clone()
                    },
                )
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
}
