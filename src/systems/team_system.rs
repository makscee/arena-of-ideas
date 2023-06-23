use super::*;

pub struct TeamSystem;

impl TeamSystem {
    pub fn try_get_state<'a>(
        faction: &Faction,
        world: &'a legion::World,
    ) -> Option<&'a ContextState> {
        if let Some(entity) = Self::entity(faction, world) {
            ContextState::try_get(entity, world)
        } else {
            None
        }
    }

    pub fn get_state<'a>(faction: &Faction, world: &'a legion::World) -> &'a ContextState {
        Self::try_get_state(faction, world)
            .expect(&format!("Failed to find team entity for {faction}"))
    }

    pub fn get_state_mut<'a>(
        faction: &Faction,
        world: &'a mut legion::World,
    ) -> &'a mut ContextState {
        ContextState::get_mut(
            Self::entity(faction, world)
                .expect(&format!("Failed to find team entity for {faction}")),
            world,
        )
    }

    pub fn entity(faction: &Faction, world: &legion::World) -> Option<legion::Entity> {
        <(&EntityComponent, &ContextState)>::query()
            .filter(component::<TeamComponent>())
            .iter(world)
            .find(|(_, state)| state.get_faction(&VarName::Faction, world) == *faction)
            .map(|x| x.0.entity)
    }

    pub fn change_slots(delta: i32, faction: &Faction, world: &mut legion::World) {
        Self::get_state_mut(faction, world)
            .vars
            .change_int(&VarName::Slots, delta);
    }
}
