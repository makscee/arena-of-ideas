use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Team {
    pub units: Vec<PackedUnit>,
    #[serde(default)]
    pub state: TeamState,
}

impl Team {
    pub fn new(name: String, units: Vec<PackedUnit>) -> Self {
        Self {
            units,
            state: TeamState::new(name),
            ..default()
        }
    }

    pub fn unpack(&self, faction: &Faction, world: &mut legion::World, resources: &mut Resources) {
        for (slot, unit) in self.units.iter().enumerate() {
            unit.unpack(world, resources, slot + 1, *faction, None);
        }
        resources
            .team_states
            .set_team_state(*faction, self.state.clone());
        debug!("Unpack team {} {:?}", self, self.state);
    }

    pub fn pack(faction: &Faction, world: &legion::World, resources: &Resources) -> Team {
        let units = UnitSystem::collect_faction_units(world, *faction)
            .into_iter()
            .sorted_by_key(|(_, unit)| unit.slot)
            .map(|(entity, _)| PackedUnit::pack(entity, world, resources))
            .collect_vec();
        let state = resources.team_states.get_team_state(faction).clone();
        Team { units, state }
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = self.units.iter().map(|x| x.to_string()).join(", ");
        write!(f, "{}[{}]", self.state.name, text)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct TeamState {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default)]
    pub vars: Vars,
    #[serde(default)]
    pub ability_overrides: HashMap<AbilityName, Vars>,
}

impl TeamState {
    pub fn new(name: String) -> Self {
        Self { name, ..default() }
    }
}

fn default_name() -> String {
    "no_name".to_string()
}
