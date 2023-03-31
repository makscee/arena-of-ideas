use super::*;

#[derive(Default)]
pub struct AbilityPool {
    effects: HashMap<AbilityName, EffectWrapped>,
    house_origin: HashMap<AbilityName, HouseName>,
    default_vars: HashMap<AbilityName, Vars>,
}

impl AbilityPool {
    pub fn define_ability(
        resources: &mut Resources,
        name: &AbilityName,
        ability: &Ability,
        color: Rgba<f32>,
        origin: HouseName,
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

    pub fn get_effect(resources: &Resources, name: &AbilityName) -> EffectWrapped {
        resources.ability_pool.effects.get(name).unwrap().clone()
    }

    pub fn get_house_origin(resources: &Resources, name: &AbilityName) -> HouseName {
        *resources.ability_pool.house_origin.get(name).unwrap()
    }

    pub fn get_default_vars(resources: &Resources, name: &AbilityName) -> Vars {
        resources
            .ability_pool
            .default_vars
            .get(name)
            .cloned()
            .unwrap_or_default()
    }

    pub fn set_var(
        resources: &mut Resources,
        faction: &Faction,
        ability: &AbilityName,
        var: VarName,
        value: Var,
    ) {
        resources
            .team_states
            .get_ability_vars_mut(faction, ability)
            .insert(var, value);
    }

    pub fn get_var(
        resources: &Resources,
        faction: &Faction,
        ability: &AbilityName,
        var: &VarName,
    ) -> Var {
        match resources
            .team_states
            .try_get_ability_overrides(faction, ability)
        {
            Some(vars) => vars.get(var).clone(),
            None => match resources
                .ability_pool
                .default_vars
                .get(ability)
                .and_then(|x| Some(x.get(var).clone()))
            {
                Some(var) => var,
                None => panic!("Var {} not set default for {:?}", var, ability),
            },
        }
    }

    pub fn set_var_int(
        resources: &mut Resources,
        faction: &Faction,
        ability: &AbilityName,
        var: VarName,
        value: i32,
    ) {
        Self::set_var(resources, faction, ability, var, Var::Int(value))
    }

    pub fn get_var_int(
        resources: &Resources,
        faction: &Faction,
        ability: &AbilityName,
        var: &VarName,
    ) -> i32 {
        match Self::get_var(resources, faction, ability, var) {
            Var::Int(value) => value,
            _ => panic!("Wrong var type"),
        }
    }
}
