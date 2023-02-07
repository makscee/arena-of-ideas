use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Debug)]
pub struct UnitComponent {}

const STATUSES_EFFECTS_KEY: &str = "statuses";

impl UnitComponent {
    pub fn add_all_units_to_node_template(world: &legion::World, resources: &mut Resources) {
        resources
            .cassette
            .node_template
            .clear_key(STATUSES_EFFECTS_KEY);
        <(&UnitComponent, &EntityComponent)>::query()
            .iter(world)
            .for_each(|(unit, entity)| {
                resources.cassette.node_template.add_entity_shader(
                    entity.entity,
                    ShaderSystem::get_entity_shader(world, entity.entity).clone(),
                );
                resources.cassette.node_template.add_effects_by_key(
                    STATUSES_EFFECTS_KEY,
                    resources
                        .statuses
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
        StatsUiSystem::add_all_units_stats_to_node_template(world, resources);
    }
}
