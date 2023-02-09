use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Debug)]
pub struct UnitComponent {
    pub slot: usize,
    pub faction: Faction,
}

#[derive(Clone, Deserialize, Debug, PartialEq, Eq, Hash)]
pub enum Faction {
    Light,
    Dark,
}

impl VarsProvider for UnitComponent {
    fn extend_vars(&self, vars: &mut Vars) {
        let faction_val = match self.faction {
            Faction::Light => -1,
            Faction::Dark => 1,
        };
        vars.insert(VarName::Faction, Var::Int(faction_val));
    }
}

const STATUSES_EFFECTS_KEY: &str = "statuses";

impl UnitComponent {
    pub fn new(slot: usize, faction: Faction) -> Self {
        Self { slot, faction }
    }

    pub fn add_all_units_to_node_template(
        world: &legion::World,
        options: &Options,
        statuses: &Statuses,
        node_template: &mut CassetteNode,
    ) {
        node_template.clear_key(STATUSES_EFFECTS_KEY);
        <(&UnitComponent, &EntityComponent)>::query()
            .iter(world)
            .for_each(|(unit, entity)| {
                node_template.add_entity_shader(
                    entity.entity,
                    ShaderSystem::get_entity_shader(world, entity.entity).clone(),
                );
                node_template.add_effects_by_key(
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
        StatsUiSystem::fill_node_template(world, options, node_template);
        NameSystem::fill_node_template(world, options, node_template);
    }
}
