use super::*;

mod ability;
mod alliances;
mod condition;
mod damage_value;
mod effect;
mod factions;
mod modifier;
mod projectile;
mod status;
mod time_bomb;
mod unit;

pub use ability::*;
pub use alliances::*;
pub use condition::*;
pub use damage_value::*;
pub use effect::*;
pub use factions::*;
pub use modifier::*;
pub use projectile::*;
pub use status::*;
pub use time_bomb::*;
pub use unit::*;

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TargetFilter {
    All,
    Allies,
    Enemies,
}

impl TargetFilter {
    pub fn matches(&self, a: Faction, b: Faction) -> bool {
        match self {
            Self::Allies => a == b,
            Self::Enemies => a != b,
            Self::All => true,
        }
    }
}

pub struct Model {
    pub next_id: Id,
    pub units: Collection<Unit>,
    pub spawning_units: Collection<Unit>,
    pub dead_units: Collection<Unit>,
    pub projectiles: Collection<Projectile>,
    pub time_bombs: Collection<TimeBomb>,
    pub dead_time_bombs: Collection<TimeBomb>,
    pub config: Config,
    pub free_revives: usize,
    pub unit_templates: UnitTemplates,
}

impl Model {
    pub fn new(config: Config, unit_templates: UnitTemplates) -> Self {
        Self {
            next_id: 0,
            units: Collection::new(),
            spawning_units: Collection::new(),
            dead_units: Collection::new(),
            projectiles: Collection::new(),
            time_bombs: Collection::new(),
            dead_time_bombs: Collection::new(),
            config,
            free_revives: 0,
            unit_templates,
        }
    }
}
