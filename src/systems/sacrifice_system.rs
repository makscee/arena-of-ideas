use super::*;

pub struct SacrificeSystem;

impl SacrificeSystem {
    pub fn sacrifice_marked(world: &mut legion::World, resources: &mut Resources) {
        let units = mem::take(&mut resources.sacrifice_data.marked_units);
        debug!("Sacrifice {units:?}");
        resources.sacrifice_data.ranks_sum = 0;
        for unit in units.iter() {
            let context = Context::new(ContextLayer::Unit { entity: *unit }, world, resources)
                .set_target(*unit);
            resources.sacrifice_data.ranks_sum +=
                context.get_int(&VarName::Rank, world).unwrap() as usize;
            Effect::Kill.wrap().push(context, resources);
            ActionSystem::spin(world, resources, &mut None);
            ActionSystem::death_check(world, resources, &mut None);
            GameStateSystem::set_transition(GameState::Shop, resources);
        }
        resources.sacrifice_data.marked_units = units;
    }

    pub fn show_bonus_widget(world: &legion::World, resources: &mut Resources) {
        let value = resources.battle_data.last_difficulty
            + 1
            + resources.sacrifice_data.ranks_sum
            + resources.battle_data.last_score;
        resources.sacrifice_data.marked_units.clear();
        BonusEffectPool::load_widget(value, world, resources);
    }
}
