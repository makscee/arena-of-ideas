use super::*;

pub struct FactionsState {
    states: HashMap<Faction, FactionState>,
}

impl FactionsState {
    pub fn set_faction_state(&mut self, faction: Faction, state: FactionState) {
        self.states.insert(faction, state);
    }

    pub fn get_faction_state(&self, faction: &Faction) -> &FactionState {
        self.states.get(faction).unwrap()
    }

    pub fn remove_faction_state(&mut self, faction: &Faction) -> FactionState {
        self.states.remove(faction).unwrap()
    }

    pub fn clear(&mut self, faction: Faction) {
        self.states.insert(faction, default());
    }

    pub fn try_get_ability_overrides(
        &self,
        faction: &Faction,
        name: &AbilityName,
    ) -> Option<&Vars> {
        self.states
            .get(faction)
            .unwrap()
            .ability_overrides
            .get(name)
    }
}

impl Default for FactionsState {
    fn default() -> Self {
        let states = HashMap::from_iter(enum_iterator::all::<Faction>().map(|f| (f, default())));
        Self { states }
    }
}

#[derive(Debug)]
pub struct FactionState {
    pub ability_overrides: HashMap<AbilityName, Vars>,
    pub team_name: String,
}

impl Default for FactionState {
    fn default() -> Self {
        Self {
            ability_overrides: default(),
            team_name: String::from("no_name"),
        }
    }
}
