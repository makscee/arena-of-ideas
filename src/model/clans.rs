use geng::prelude::itertools::Itertools;

use super::*;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
pub enum Clan {
    Spawners,
    Assassins,
    Critters,
    Archers,
    Freezers,
    Warriors,
    Healers,
    Vampires,
    Exploders,
    Splashers,
    Chainers,
    Charmers,
    PlagueSpreaders,
    Skeletons,
    Warlocks,
    Protectors,
    Demons,
    Dragons,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ClanFilter(Option<HashMap<Faction, Option<Vec<Clan>>>>);

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ClanEffect {
    /// Number of heroes required to activate the effect
    activate: usize,
    /// Whether effects with lower requirements should be removed
    #[serde(default)]
    replace: bool,
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
    /// Checks the filters (factions and clans) and applies the
    /// effects if the constraints are met.
    fn apply(&self, unit: &mut Unit, next_id: &mut Id, statuses: &Statuses) {
        if !self.filter.check(unit) {
            return;
        }
        unit.all_statuses.extend(self.statuses.iter().map(|status| {
            status
                .get(statuses)
                .clone()
                .attach(Some(unit.id), None, next_id)
        }));
    }
}

impl fmt::Display for Clan {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Clan::Spawners => write!(f, "Spawners"),
            Clan::Assassins => write!(f, "Assassins"),
            Clan::Archers => write!(f, "Archers"),
            Clan::Freezers => write!(f, "Freezers"),
            Clan::Warriors => write!(f, "Warriors"),
            Clan::Healers => write!(f, "Healers"),
            Clan::Vampires => write!(f, "Vampires"),
            Clan::Critters => write!(f, "Critters"),
            Clan::Exploders => write!(f, "Exploders"),
            Clan::Splashers => write!(f, "Splashers"),
            Clan::Chainers => write!(f, "Chainers"),
            Clan::Charmers => write!(f, "Charmers"),
            Clan::PlagueSpreaders => write!(f, "PlagueSpreaders"),
            Clan::Skeletons => write!(f, "Skeletons"),
            Clan::Warlocks => write!(f, "Warlocks"),
            Clan::Protectors => write!(f, "Protectors"),
            Clan::Demons => write!(f, "Demons"),
            Clan::Dragons => write!(f, "Dragons"),
        }
    }
}

impl Clan {
    pub fn apply_effects(
        &self,
        unit: &mut Unit,
        effects: &ClanEffects,
        party_members: usize,
        next_id: &mut Id,
        statuses: &Statuses,
    ) {
        let effects = match effects.get(self) {
            Some(effects) => effects,
            None => {
                error!("Failed to find effects for the clan {self:?}");
                return;
            }
        };
        let effects = effects
            .iter()
            .filter(|effect| effect.activate <= party_members)
            .sorted_by_key(|effect| effect.activate);
        for effect in effects.rev() {
            effect.apply(unit, next_id, statuses);
            if effect.replace {
                break;
            }
        }
    }
}
