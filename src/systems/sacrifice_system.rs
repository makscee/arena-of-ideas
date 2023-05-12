use super::*;

pub struct SacrificeSystem;

impl SacrificeSystem {
    pub fn sacrifice_marked(world: &mut legion::World, resources: &mut Resources) {
        let units = mem::take(&mut resources.sacrifice_data.marked_units);
        debug!("Sacrifice {units:?}");
        for unit in units.iter() {
            Effect::Kill.wrap().push(
                Context::new(ContextLayer::Unit { entity: *unit }, world, resources)
                    .set_target(*unit),
                resources,
            );
            ActionSystem::spin(world, resources, &mut None);
            ActionSystem::death_check(world, resources, &mut None);
            GameStateSystem::set_transition(GameState::Shop, resources);
        }
        resources.sacrifice_data.marked_units = units;
    }

    pub fn show_bonus_widget(world: &legion::World, resources: &mut Resources) {
        let value = resources.sacrifice_data.marked_units.len()
            + resources.battle_data.last_difficulty
            + 1
            + resources.battle_data.last_score;
        resources.sacrifice_data.marked_units.clear();
        BonusEffectPool::load_widget(value, world, resources);
    }
}
