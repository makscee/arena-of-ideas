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
    /// Checks the filters (factions and clans) and applies the
    /// effects if the constraints are met.
    fn apply(&self, unit: &mut Unit, statuses: &Statuses, orchestrator: &mut EffectOrchestrator) {
        if !self.filter.check(unit) {
            return;
        }
        for status in self.statuses.iter() {
            let color = status.get(statuses).color;
            orchestrator.push_back(
                EffectContext {
                    queue_id: Some("Spawn".to_owned()),
                    owner: unit.id,
                    creator: unit.id,
                    target: unit.id,
                    vars: default(),
                    status_id: None,
                    color,
                },
                Effect::AttachStatus(Box::new(AttachStatusEffect {
                    status: status.clone(),
                    vars: default(),
                })),
            )
        }
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

impl Clan {
    pub fn apply_effects(
        &self,
        unit: &mut Unit,
        effects: &ClanEffects,
        party_members: usize,
        id: Id,
        statuses: &Statuses,
        orchestrator: &mut EffectOrchestrator,
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
            effect.apply(unit, statuses, orchestrator);
        }
    }
}
