use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
pub enum Clan {
    Warlocks,
    Protectors,
    Demons,
    Wizards,
    Common,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ClanFilter(Option<HashMap<Faction, Option<Vec<Clan>>>>);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ClanEffect {
    /// Number of heroes required to activate the effect
    activate: usize,
    /// Filter target units by factions and alliances
    #[serde(default)]
    filter: ClanFilter,
    /// Statuses to apply to every target unit
    statuses: Vec<StatusRef>,
}

impl ClanFilter {
    /// Checks whether the unit satisfies the filter conditions
    pub fn check(&self, unit: &Unit) -> bool {
        let filter = match &self.0 {
            None => return true,
            Some(filter) => filter,
        };
        match filter.get(&unit.faction) {
            None => false,
            Some(None) => true,
            Some(Some(clans)) => clans.iter().any(|clan| unit.clans.contains(clan)),
        }
    }
}

impl ClanEffect {
    /// Checks the filters (factions and clans) and get the
    /// statuses if the constraints are met.
    pub fn get_check(&self, unit: &mut Unit, size: usize) -> Vec<StatusRef> {
        if self.activate <= size && !self.filter.check(unit) {
            return vec![];
        }
        self.statuses.clone()
    }
}

impl fmt::Display for Clan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Clan::Warlocks => write!(f, "Warlocks"),
            Clan::Protectors => write!(f, "Protectors"),
            Clan::Demons => write!(f, "Demons"),
            Clan::Wizards => write!(f, "Wizards"),
            Clan::Common => write!(f, "Common"),
        }
    }
}
