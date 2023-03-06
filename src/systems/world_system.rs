use super::*;

pub struct WorldSystem {}

impl WorldSystem {
    pub fn get_context(world: &legion::World) -> Context {
        <(&WorldComponent, &Context)>::query()
            .iter(world)
            .collect_vec()[0]
            .1
            .clone()
    }

    pub fn set_var(world: &mut legion::World, name: VarName, value: &Var) {
        <(&WorldComponent, &mut Context)>::query()
            .iter_mut(world)
            .for_each(|(_, context)| context.vars.insert(name, value.clone()));
    }

    pub fn get_var_float(world: &legion::World, name: &VarName) -> f32 {
        let var = Self::get_vars(world).get(name);
        match var {
            Var::Float(value) => *value,
            _ => panic!("Wrong var type {:?}", var),
        }
    }

    pub fn get_var_vec2(world: &legion::World, name: &VarName) -> vec2<f32> {
        let var = Self::get_vars(world).get(name);
        match var {
            Var::Vec2(value) => *value,
            _ => panic!("Wrong var type {:?}", var),
        }
    }

    pub fn get_vars(world: &legion::World) -> &Vars {
        &<(&WorldComponent, &Context)>::query()
            .iter(world)
            .collect_vec()[0]
            .1
            .vars
    }

    pub fn set_time(ts: Time, world: &mut legion::World) {
        <&mut WorldComponent>::query().iter_mut(world).collect_vec()[0].global_time = ts;
    }

    pub fn get_time(world: &legion::World) -> Time {
        <&WorldComponent>::query().iter(world).collect_vec()[0].global_time
    }
}
