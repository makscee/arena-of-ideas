use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Team {
    pub units: Vec<PackedUnit>,
    #[serde(default)]
    pub state: TeamState,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ReplicatedTeam {
    #[serde(flatten)]
    pub team: Team,
    #[serde(default)]
    pub replications: usize,
}

impl From<ReplicatedTeam> for Team {
    fn from(value: ReplicatedTeam) -> Self {
        let mut team = value.team;
        let size = team.units.len();
        for _ in 1..value.replications {
            for i in 0..size {
                team.units.push(team.units[i].clone())
            }
        }
        team.state.slots = team.units.len();
        team
    }
}

impl Team {
    pub fn new(name: String, units: Vec<PackedUnit>) -> Self {
        Self {
            units,
            state: TeamState::new(name),
            ..default()
        }
    }

    pub fn unpack(
        &self,
        faction: &Faction,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> Vec<legion::Entity> {
        let mut entities = vec![];
        for (slot, unit) in self.units.iter().enumerate() {
            entities.push(unit.unpack(world, resources, slot + 1, *faction, None));
        }
        resources
            .team_states
            .set_team_state(*faction, self.state.clone());
        debug!(
            "Unpack team {} {:?} {}",
            self,
            self.state,
            entities.iter().map(|x| format!("{:?}", x)).join(", ")
        );
        entities
    }

    pub fn pack(faction: &Faction, world: &legion::World, resources: &Resources) -> Team {
        let units = UnitSystem::collect_faction_units(world, resources, *faction, false)
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

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
pub struct TeamState {
    pub name: String,
    pub vars: Vars,
    pub ability_overrides: HashMap<AbilityName, Vars>,
    pub slots: usize,
}

impl Default for TeamState {
    fn default() -> Self {
        Self {
            name: String::from("no_name"),
            vars: Default::default(),
            ability_overrides: Default::default(),
            slots: MAX_SLOTS,
        }
    }
}

impl TeamState {
    pub fn new(name: String) -> Self {
        Self { name, ..default() }
    }
}
