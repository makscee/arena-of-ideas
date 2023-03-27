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
        let mut state = resources.factions_state.remove_faction_state(faction);
        let mut vars = state.ability_overrides.remove(ability).unwrap_or_default();
        vars.insert(var, value);
        state.ability_overrides.insert(*ability, vars);
        resources.factions_state.set_faction_state(*faction, state);
    }

    pub fn get_var(
        resources: &Resources,
        faction: &Faction,
        ability: &AbilityName,
        var: &VarName,
    ) -> Var {
        resources
            .factions_state
            .get_faction_state(faction)
            .ability_overrides
            .get(ability)
            .and_then(|x| x.try_get(var).cloned())
            .unwrap_or_else(|| Self::get_default_vars(resources, ability).get(var).clone())
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
