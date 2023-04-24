use super::*;

#[derive(Default)]
pub struct AbilityPool {
    effects: HashMap<AbilityName, EffectWrapped>,
    house_origin: HashMap<AbilityName, HouseName>,
    default_vars: HashMap<AbilityName, Vars>,
}

impl AbilityPool {
    pub fn define_ability(
        name: &AbilityName,
        ability: &Ability,
        color: Rgba<f32>,
        origin: HouseName,
        resources: &mut Resources,
    ) {
        resources
            .definitions
            .insert(name.to_string(), color, ability.description.clone());

        let pool = &mut resources.ability_pool;
        pool.house_origin.insert(*name, origin);
        pool.effects.insert(*name, ability.effect.clone());
        let mut vars = ability.default_vars.clone();
        vars.insert(VarName::Color, Var::Color(color));
        pool.default_vars.insert(*name, vars);
    }

    pub fn init_world(world: &mut legion::World, resources: &Resources) {
        for (name, vars) in resources.ability_pool.default_vars.iter() {
            WorldSystem::get_state_mut(world)
                .ability_vars
                .insert(*name, vars.clone());
        }
    }

    pub fn get_effect(resources: &Resources, name: &AbilityName) -> EffectWrapped {
        resources.ability_pool.effects.get(name).unwrap().clone()
    }

    pub fn get_house_origin(resources: &Resources, name: &AbilityName) -> HouseName {
        *resources.ability_pool.house_origin.get(name).unwrap()
    }
}
