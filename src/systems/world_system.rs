use super::*;

pub struct WorldSystem {}

impl WorldSystem {
    pub fn init_world_entity(world: &mut legion::World) -> legion::Entity {
        let world_entity = world.push((WorldComponent {},));
        let mut world_entry = world.entry(world_entity).unwrap();
        world_entry.add_component(EntityComponent {
            entity: world_entity,
        });
        let mut vars = Vars::default();
        vars.insert(VarName::FieldPosition, Var::Vec2(vec2(0.0, 0.0)));
        world_entry.add_component(Context {
            owner: world_entity,
            target: world_entity,
            parent: None,
            vars,
        });
        world_entity
    }

    pub fn get_context(world: &legion::World) -> Context {
        <(&WorldComponent, &Context)>::query()
            .iter(world)
            .collect_vec()[0]
            .1
            .clone()
    }

    pub fn set_var(world: &mut legion::World, name: VarName, value: Var) {
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
        Self::set_var(world, VarName::GlobalTime, Var::Float(ts));
    }

    pub fn get_time(world: &legion::World) -> Time {
        Self::get_var_float(world, &VarName::GlobalTime)
    }
}
