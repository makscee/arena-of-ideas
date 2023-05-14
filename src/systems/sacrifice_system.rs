use super::*;

pub struct SacrificeSystem;

impl SacrificeSystem {
    pub fn sacrifice_marked(world: &mut legion::World, resources: &mut Resources) {
        let units = mem::take(&mut resources.sacrifice_data.marked_units);
        debug!("Sacrifice {units:?}");
        let mut sum = 0;
        for unit in units.iter() {
            let context = Context::new(ContextLayer::Unit { entity: *unit }, world, resources)
                .set_target(*unit);
            sum += context.get_int(&VarName::Rank, world).unwrap();
            Effect::Kill.wrap().push(context, resources);
            ActionSystem::spin(world, resources, &mut None);
            ActionSystem::death_check(world, resources, &mut None);
            GameStateSystem::set_transition(GameState::Shop, resources);
        }
        TeamSystem::get_state_mut(&Faction::Team, world)
            .vars
            .change_int(&VarName::Stars, sum);
    }

    pub fn show_bonus_widget(world: &legion::World, resources: &mut Resources) {
        let value =
            TeamSystem::get_state(&Faction::Team, world).get_int(&VarName::Stars, world) as usize;
        resources.sacrifice_data.marked_units.clear();
        BonusEffectPool::load_widget(value, world, resources);
    }
}
