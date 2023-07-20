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
            sum += context.get_int(&VarName::Rank, world).unwrap() + 1;
            Effect::Kill.wrap().push(context, resources);
            ActionSystem::spin(world, resources, None);
            ActionSystem::death_check(world, resources, None);
        }
        ShopSystem::change_g(sum, Some("Sacrifice"), world, resources);
        GameStateSystem::set_transition(GameState::Shop, resources);
    }
}
