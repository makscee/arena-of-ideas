use std::collections::VecDeque;

use super::*;

mod ability;
mod clans;
mod condition;
mod effect;
mod expr;
mod factions;
mod modifier;
mod particle;
mod position;
mod projectile;
mod render;
mod status;
mod time_bomb;
mod unit;

pub use ability::*;
pub use clans::*;
pub use condition::*;
pub use effect::*;
pub use expr::*;
pub use factions::*;
pub use modifier::*;
pub use particle::*;
pub use position::*;
pub use projectile::*;
pub use render::*;
pub use status::*;
pub use time_bomb::*;
pub use unit::*;

// TODO: make configurable
pub const SIDE_SLOTS: usize = 5;

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

#[derive(Clone)]
pub struct Model {
    pub next_id: Id,
    pub time: Time,
    pub units: Collection<Unit>,
    pub spawning_units: Collection<Unit>,
    pub dead_units: Collection<Unit>,
    pub projectiles: Collection<Projectile>,
    pub time_bombs: Collection<TimeBomb>,
    pub dead_time_bombs: Collection<TimeBomb>,
    pub particles: Collection<Particle>,
    pub config: Config,
    pub round: GameRound,
    pub wave_delay: Time,
    pub free_revives: usize,
    pub unit_templates: UnitTemplates,
    pub clan_effects: ClanEffects,
    pub statuses: Statuses,
    pub delayed_effects: std::collections::BinaryHeap<QueuedEffect<DelayedEffect>>,
    pub transition: bool,
    /// Variables that persist for the whole game
    pub vars: HashMap<VarName, R32>,
}

impl Model {
    pub fn new(
        config: Config,
        unit_templates: UnitTemplates,
        clan_effects: ClanEffects,
        statuses: Statuses,
        round: GameRound,
    ) -> Self {
        Self {
            next_id: 0,
            time: Time::ZERO,
            units: Collection::new(),
            spawning_units: Collection::new(),
            dead_units: Collection::new(),
            projectiles: Collection::new(),
            time_bombs: Collection::new(),
            dead_time_bombs: Collection::new(),
            particles: Collection::new(),
            wave_delay: Time::ZERO,
            free_revives: 0,
            unit_templates,
            clan_effects,
            statuses,
            delayed_effects: default(),
            transition: false,
            round,
            config,
            vars: HashMap::new(),
        }
    }
}
