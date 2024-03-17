use crate::module_bindings::TableUnit;

use super::*;

#[derive(Deserialize, Serialize, TypeUuid, TypePath, Debug, Clone, Default)]
#[uuid = "cb5457bc-b429-4af8-8d92-bf141a80020b"]
pub struct PackedTeam {
    pub units: Vec<PackedUnit>,
    #[serde(default)]
    pub state: VarState,
    #[serde(default)]
    pub ability_states: HashMap<String, VarState>,
}

#[derive(Component, Default, Clone, Debug)]
pub struct AbilityStates(pub HashMap<String, VarState>);

impl PackedTeam {
    pub fn new(units: Vec<PackedUnit>) -> Self {
        Self {
            units,
            state: default(),
            ability_states: default(),
        }
    }
    pub fn from_table_units(units: Vec<TableUnit>) -> Self {
        Self::new(units.into_iter().map(|u| u.into()).collect())
    }
    pub fn unpack(self, faction: Faction, world: &mut World) {
        let team = TeamPlugin::spawn(faction, world);
        self.state.attach(team, world);
        world
            .entity_mut(team)
            .insert(AbilityStates(self.ability_states));
        for (i, unit) in self.units.into_iter().enumerate() {
            unit.unpack(team, Some(i + 1), world);
        }
    }
    pub fn pack(faction: Faction, world: &mut World) -> Self {
        let team = TeamPlugin::find_entity(faction, world).unwrap();
        let state = VarState::get(team, world).clone();
        let ability_states = world.get::<AbilityStates>(team).unwrap().0.clone();
        let units = UnitPlugin::collect_factions(HashSet::from([faction]), world)
            .into_iter()
            .map(|(u, _)| PackedUnit::pack(u, world))
            .sorted_by_key(|u| u.state.get_int(VarName::Slot).unwrap_or_default())
            .collect_vec();
        PackedTeam {
            units,
            state,
            ability_states,
        }
    }
}

impl ToString for PackedTeam {
    fn to_string(&self) -> String {
        let mut result = String::with_capacity(30);
        let mut i = 0;
        while i < self.units.len() {
            if !result.is_empty() {
                result.push_str(", ");
            }
            let name = self.units[i].name.clone();
            let statuses = self.units[i].statuses_string();
            let mut count = 1;
            for c in i + 1..self.units.len() {
                count = c - i + 1;
                if !self.units[c].name.eq(&name) || !self.units[c].statuses_string().eq(&statuses) {
                    break;
                }
            }
            if count > 1 {
                result.push_str(&format!("{name} x{count}"));
            } else {
                result.push_str(&name.to_string());
            }
            if !statuses.is_empty() {
                result.push_str(&format!(" with {statuses}"));
            }
            i += count;
        }
        result
    }
}
