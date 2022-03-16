use super::*;

mod ability;
mod alliances;
mod damage_value;
mod effect;
mod factions;
mod projectile;
mod status;
mod time_bomb;
mod unit;

pub use ability::*;
pub use alliances::*;
pub use damage_value::*;
pub use effect::*;
pub use factions::*;
pub use projectile::*;
pub use status::*;
pub use time_bomb::*;
pub use unit::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TargetFilter {
    All,
    Allies,
    Enemies,
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
            unit_templates,
        }
    }
}
