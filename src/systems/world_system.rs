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

    pub fn collect_factions(
        world: &legion::World,
        factions: HashSet<Faction>,
    ) -> HashMap<legion::Entity, UnitComponent> {
        HashMap::from_iter(
            <(&UnitComponent, &EntityComponent)>::query()
                .iter(world)
                .filter_map(|(unit, entity)| match factions.contains(&unit.faction) {
                    true => Some((entity.entity, *unit)),
                    false => None,
                }),
        )
    }

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

    pub fn kill(
        entity: legion::Entity,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> bool {
        let unit = world
            .entry_ref(entity)
            .unwrap()
            .get_component::<UnitComponent>()
            .unwrap()
            .clone();
        if unit.faction == Faction::Team {
            Event::RemoveFromTeam {}.send(&Context::construct_context(&entity, world), resources);
        }
        world.remove(entity)
    }

    pub fn set_time(ts: Time, world: &mut legion::World) {
        <&mut WorldComponent>::query().iter_mut(world).collect_vec()[0].global_time = ts;
    }

    pub fn get_time(world: &legion::World) -> Time {
        <&WorldComponent>::query().iter(world).collect_vec()[0].global_time
    }
}
