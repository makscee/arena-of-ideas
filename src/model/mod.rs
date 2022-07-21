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
pub use render::*;
pub use status::*;
pub use time_bomb::*;
pub use unit::*;

// TODO: make configurable
pub const SIDE_SLOTS: usize = 5;
pub const TICK_TIME: f32 = 2.0;

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
pub struct TickModel {
    pub tick_time: Time,
    /// Queue of units to perform actions.
    /// Some units may die before the queue is reset.
    pub action_queue: VecDeque<Id>,
    pub current_action_time_left: Time,
}

#[derive(Clone)]
pub struct Model {
    pub next_id: Id,
    pub time: Time,
    pub units: Collection<Unit>,
    pub dead_units: Collection<Unit>,
    pub time_bombs: Collection<TimeBomb>,
    pub dead_time_bombs: Collection<TimeBomb>,
    pub particles: Collection<Particle>,
    pub config: Config,
    pub round: Option<GameRound>,
    pub free_revives: usize,
    pub unit_templates: UnitTemplates,
    pub clan_effects: ClanEffects,
    pub statuses: Statuses,
    pub delayed_effects: std::collections::BinaryHeap<QueuedEffect<DelayedEffect>>,
    pub transition: bool,
    /// Variables that persist for the whole game
    pub vars: HashMap<VarName, R32>,
    pub current_tick: TickModel,
    pub current_tick_num: usize,
}

impl Model {
    pub fn new(
        config: Config,
        unit_templates: UnitTemplates,
        clan_effects: ClanEffects,
        statuses: Statuses,
        round: Option<GameRound>,
    ) -> Self {
        Self {
            next_id: 0,
            time: Time::ZERO,
            units: Collection::new(),
            dead_units: Collection::new(),
            time_bombs: Collection::new(),
            dead_time_bombs: Collection::new(),
            particles: Collection::new(),
            free_revives: 0,
            unit_templates,
            clan_effects,
            statuses,
            delayed_effects: default(),
            transition: false,
            round,
            config,
            vars: HashMap::new(),
            current_tick: TickModel::new(),
            current_tick_num: 0,
        }
    }
}

impl TickModel {
    pub fn new() -> Self {
        Self {
            tick_time: Time::ZERO,
            action_queue: VecDeque::new(),
            current_action_time_left: Time::ZERO,
        }
    }
}
