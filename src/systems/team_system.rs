use super::*;

pub struct TeamSystem;

impl TeamSystem {
    pub fn get_state<'a>(faction: &Faction, world: &'a legion::World) -> &'a ContextState {
        ContextState::get(Self::entity(faction, world).unwrap(), world)
    }

    pub fn get_state_mut<'a>(
        faction: &Faction,
        world: &'a mut legion::World,
    ) -> &'a mut ContextState {
        ContextState::get_mut(Self::entity(faction, world).unwrap(), world)
    }

    pub fn entity(faction: &Faction, world: &legion::World) -> Option<legion::Entity> {
        <(&EntityComponent, &ContextState)>::query()
            .filter(component::<TeamComponent>())
            .iter(world)
            .find(|(_, state)| state.get_faction(&VarName::Faction, world) == *faction)
            .map(|x| x.0.entity)
    }
}
