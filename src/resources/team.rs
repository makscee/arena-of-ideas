use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Team {
    #[serde(default = "default_name")]
    pub name: String,
    pub units: Vec<PackedUnit>,
    #[serde(default)]
    pub ability_overrides: HashMap<AbilityName, Vars>,
}
fn default_name() -> String {
    "no_name".to_string()
}

impl Team {
    pub fn new(name: String, units: Vec<PackedUnit>) -> Self {
        Self {
            name,
            units,
            ..default()
        }
    }

    pub fn unpack(&self, faction: &Faction, world: &mut legion::World, resources: &mut Resources) {
        for (slot, unit) in self.units.iter().enumerate() {
            unit.unpack(world, resources, slot + 1, *faction, None);
        }
        resources.factions_state.set_faction_state(
            *faction,
            FactionState {
                ability_overrides: self.ability_overrides.clone(),
                team_name: self.name.clone(),
            },
        );
    }

    pub fn pack(faction: &Faction, world: &legion::World, resources: &Resources) -> Team {
        let units = UnitSystem::collect_faction_units(world, *faction)
            .into_iter()
            .sorted_by_key(|(_, unit)| unit.slot)
            .map(|(entity, _)| PackedUnit::pack(entity, world, resources))
            .collect_vec();
        let state = resources.factions_state.get_faction_state(faction);
        Team {
            name: state.team_name.clone(),
            units,
            ability_overrides: state.ability_overrides.clone(),
        }
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = self.units.iter().map(|x| x.to_string()).join(", ");
        write!(f, "{}[{}]", self.name, text)
    }
}
