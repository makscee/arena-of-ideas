use super::*;

pub struct WorldSystem {}

impl WorldSystem {
    pub fn clear_factions(world: &mut legion::World, factions: HashSet<Faction>) {
        let unit_entitites = <(&EntityComponent, &UnitComponent)>::query()
            .iter(world)
            .filter_map(|(entity, unit)| match factions.contains(&unit.faction) {
                true => Some(entity.entity.clone()),
                false => None,
            })
            .collect_vec();
        unit_entitites.iter().for_each(|entity| {
            world.remove(*entity);
        });
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

    pub fn get_vars(world: &legion::World) -> &Vars {
        &<(&WorldComponent, &Context)>::query()
            .iter(world)
            .collect_vec()[0]
            .1
            .vars
    }
}
