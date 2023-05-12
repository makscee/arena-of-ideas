use super::*;

pub struct WorldSystem {}

impl WorldSystem {
    pub fn init_world_entity(world: &mut legion::World, options: &Options) -> legion::Entity {
        let world_entity = world.push((WorldComponent {},));
        let mut world_entry = world.entry(world_entity).unwrap();
        world_entry.add_component(EntityComponent::new(world_entity));
        let mut vars = Vars::default();
        vars.insert(VarName::FieldPosition, Var::Vec2(vec2(0.0, 0.0)));
        vars.set_color(&VarName::BackgroundLight, options.colors.background_light);
        vars.set_color(&VarName::BackgroundDark, options.colors.background_dark);
        vars.set_color(&VarName::OutlineColor, options.colors.outline);
        world_entry.add_component(ContextState {
            statuses: default(),
            ability_vars: default(),
            vars,
            parent: None,
            name: "World".to_owned(),
            status_change_t: default(),
            t: default(),
        });
        world_entity
    }

    pub fn get_state<'a>(world: &'a legion::World) -> &'a ContextState {
        if let Ok(entry) = world.entry_ref(Self::entity(world)) {
            if let Ok(state) = entry.into_component::<ContextState>() {
                return state;
            }
        }
        panic!("World state absent")
    }

    pub fn get_state_mut<'a>(world: &'a mut legion::World) -> &'a mut ContextState {
        if let Ok(entry) = world.entry_mut(Self::entity(world)) {
            if let Ok(state) = entry.into_component_mut::<ContextState>() {
                return state;
            }
        }
        panic!("World state absent")
    }

    pub fn entity(world: &legion::World) -> legion::Entity {
        <(&WorldComponent, &EntityComponent)>::query()
            .iter(world)
            .collect_vec()[0]
            .1
            .entity
    }
}
