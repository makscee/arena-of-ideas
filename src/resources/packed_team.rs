use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct PackedTeam {
    pub units: Vec<PackedUnit>,
    #[serde(default = "default_team_name")]
    pub name: String,
    #[serde(default)]
    pub vars: Vars,
    #[serde(default)]
    pub ability_vars: HashMap<AbilityName, Vars>,
    #[serde(default)]
    pub statuses: Vec<(String, i32)>,
    #[serde(default = "default_team_slots")]
    pub slots: usize,
}

fn default_team_name() -> String {
    "unnamed".to_owned()
}

fn default_team_slots() -> usize {
    3
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct ReplicatedTeam {
    #[serde(flatten)]
    pub team: PackedTeam,
    #[serde(default)]
    pub replications: usize,
}

impl From<ReplicatedTeam> for PackedTeam {
    fn from(value: ReplicatedTeam) -> Self {
        let mut team = value.team;
        let size = team.units.len();
        for _ in 1..value.replications {
            for i in 0..size {
                team.units.push(team.units[i].clone())
            }
        }
        team.slots = team.units.len();
        team
    }
}

impl PackedTeam {
    pub fn new(name: String, units: Vec<PackedUnit>) -> Self {
        Self {
            units,
            name,
            vars: default(),
            ability_vars: default(),
            statuses: default(),
            slots: default(),
        }
    }

    pub fn from_units(units: Vec<PackedUnit>, slots: Option<usize>) -> Self {
        let mut team = Self::new("".to_owned(), units);
        team.generate_name();
        let slots = match slots {
            Some(v) => v,
            None => team.units.len(),
        };
        team.slots = slots;
        team
    }

    pub fn unpack(
        &self,
        faction: &Faction,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> legion::Entity {
        let mut entities = vec![];
        let mut state = ContextState::new(self.name.clone(), Some(WorldSystem::entity(world)));
        state.ability_vars = self.ability_vars.clone();
        state.vars = self.vars.clone();
        StatusSystem::unpack_into_state(&mut state, &self.statuses);
        state.vars.set_int(&VarName::Slots, self.slots as i32);
        state.vars.set_faction(&VarName::Faction, *faction);
        let team = world.push((TeamComponent {},));
        resources.logger.log(
            || {
                format!(
                    "Unpack team {self} ({team:?}); units: {} state: {state:?}",
                    entities.iter().map(|x| format!("{:?}", x)).join(", ")
                )
            },
            &LogContext::UnitCreation,
        );
        let mut entry = world.entry(team).unwrap();
        entry.add_component(EntityComponent::new(team));
        entry.add_component(state);
        for (slot, unit) in self.units.iter().enumerate() {
            entities.push(unit.unpack(world, resources, slot + 1, None, Some(team)));
        }
        team
    }

    pub fn pack(faction: Faction, world: &legion::World, resources: &Resources) -> PackedTeam {
        let units = UnitSystem::collect_faction_states(world, faction)
            .iter()
            .sorted_by_key(|(_, state)| state.vars.get_int(&VarName::Slot))
            .map(|(entity, _)| PackedUnit::pack(*entity, world, resources))
            .collect_vec();
        let state = TeamSystem::get_state(faction, world);
        let name = state.name.clone();
        let vars = state.vars.clone();
        let ability_vars = state.ability_vars.clone();
        let statuses = StatusSystem::pack_state_into_vec(state);
        let slots = state.vars.get_int(&VarName::Slots) as usize;
        PackedTeam {
            units,
            name,
            vars,
            ability_vars,
            statuses,
            slots,
        }
    }

    pub fn generate_name(&mut self) {
        self.name = self.units.iter().map(|x| x.name.clone()).join(", ");
    }
}

impl fmt::Display for PackedTeam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let text = self.units.iter().map(|x| x.to_string()).join(", ");
        write!(f, "{}[{}]", self.name, text)
    }
}
