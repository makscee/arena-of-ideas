use geng::prelude::itertools::Itertools;

use super::*;

pub struct WorldComponent {}

impl WorldComponent {
    pub fn add_var(world: &mut legion::World, name: VarName, value: &Var) {
        <(&WorldComponent, &mut Context)>::query()
            .iter_mut(world)
            .for_each(|(_, context)| context.vars.insert(name, value.clone()));
    }

    pub fn get_vars(world: &legion::World) -> &Vars {
        &<(&WorldComponent, &Context)>::query()
            .iter(world)
            .collect_vec()[0]
            .1
            .vars
    }
}
