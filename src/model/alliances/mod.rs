use geng::prelude::itertools::Itertools;

use super::*;

mod archers;
mod assassins;
mod chainers;
mod charmers;
mod critters;
mod exploders;
mod freezers;
mod healers;
mod spawners;
mod splashers;
mod vampires;
mod warriors;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Copy, Clone, PartialOrd, Ord)]
pub enum Alliance {
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
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AllianceEffect {
    /// Number of heroes required to activate the effect
    activate: usize,
    /// Whether effects with lower requirements should be removed
    #[serde(default)]
    replace: bool,
    /// Filter target units by factions
    #[serde(default)]
    factions: Option<Vec<Faction>>,
    /// Filter target units by clan
    #[serde(default)]
    clans: Option<Vec<Alliance>>,
    /// Statuses to apply to every target unit
    statuses: Vec<Status>,
}

impl AllianceEffect {
    fn apply(&self, unit: &mut Unit) {
        todo!()
    }
}

impl Alliance {
    pub fn apply_effects(&self, unit: &mut Unit, effects: &AllianceEffects, party_members: usize) {
        let effects = effects
            .get(self)
            .expect(&format!("Failed to find effects for the alliance {self:?}"));
        let effects = effects
            .iter()
            .filter(|effect| effect.activate <= party_members)
            .sorted_by_key(|effect| effect.activate);
        for effect in effects {
            effect.apply(unit);
            if effect.replace {
                break;
            }
        }
    }
}
