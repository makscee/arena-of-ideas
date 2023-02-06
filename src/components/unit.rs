use super::*;

#[derive(Debug)]
pub struct UnitComponent {}

impl UnitComponent {
    pub fn add_all_units_to_node_template(world: &legion::World, resources: &mut Resources) {
        <(&UnitComponent, &EntityComponent)>::query()
            .iter(world)
            .for_each(|(unit, entity)| {
                resources.cassette.node_template.add_entity_shader(
                    entity.entity,
                    ShaderSystem::get_entity_shader(world, entity.entity).clone(),
                );
            });
        StatsUiSystem::add_all_units_stats_to_node_template(world, resources);
    }
}
