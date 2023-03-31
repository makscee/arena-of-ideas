use super::*;

pub struct TeamStates {
    states: HashMap<Faction, TeamState>,
}

impl TeamStates {
    pub fn set_team_state(&mut self, faction: Faction, state: TeamState) {
        self.states.insert(faction, state);
    }

    pub fn get_team_state(&self, faction: &Faction) -> &TeamState {
        self.states.get(faction).unwrap()
    }

    pub fn get_team_state_mut(&mut self, faction: &Faction) -> &mut TeamState {
        self.states.get_mut(faction).unwrap()
    }

    pub fn get_ability_vars_mut(&mut self, faction: &Faction, ability: &AbilityName) -> &mut Vars {
        if !self
            .states
            .get(faction)
            .unwrap()
            .ability_overrides
            .contains_key(ability)
        {
            self.states
                .get_mut(faction)
                .unwrap()
                .ability_overrides
                .insert(*ability, default());
        }
        self.get_team_state_mut(faction)
            .ability_overrides
            .get_mut(ability)
            .unwrap()
    }

    pub fn try_get_ability_overrides(
        &self,
        faction: &Faction,
        ability: &AbilityName,
    ) -> Option<&Vars> {
        self.states
            .get(faction)
            .unwrap()
            .ability_overrides
            .get(ability)
    }

    pub fn remove_team_state(&mut self, faction: &Faction) -> TeamState {
        self.states.remove(faction).unwrap()
    }

    pub fn clear(&mut self, faction: Faction) {
        self.states.insert(faction, default());
    }

    pub fn get_vars(&self, faction: &Faction) -> &Vars {
        &Self::get_team_state(&self, faction).vars
    }

    pub fn get_vars_mut(&mut self, faction: &Faction) -> &mut Vars {
        &mut self.get_team_state_mut(faction).vars
    }

    pub fn set_var(&mut self, faction: &Faction, var: VarName, value: Var) {
        self.get_vars_mut(faction).insert(var, value)
    }
}

impl Default for TeamStates {
    fn default() -> Self {
        let states = HashMap::from_iter(enum_iterator::all::<Faction>().map(|f| (f, default())));
        Self { states }
    }
}
