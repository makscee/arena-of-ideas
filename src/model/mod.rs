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
pub use unit::*;

// TODO: make configurable
pub const SIDE_SLOTS: usize = 5;
pub const UNIT_VISUAL_TIME: f32 = 0.5;

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
    pub tick_num: usize,
    pub visual_timer: Time,
}

#[derive(Clone)]
pub struct Model {
    pub next_id: Id,
    pub time: Time,
    pub units: Collection<Unit>,
    pub dead_units: Collection<Unit>,
    pub particles: Collection<Particle>,
    pub config: Config,
    pub round: GameRound,
    pub unit_templates: UnitTemplates,
    pub clan_effects: ClanEffects,
    pub statuses: Statuses,
    pub transition: bool,
    pub render_model: RenderModel,
    /// Variables that persist for the whole game
    pub vars: HashMap<VarName, R32>,
    pub current_tick: TickModel,
}

impl Model {
    pub fn new(
        config: Config,
        unit_templates: UnitTemplates,
        clan_effects: ClanEffects,
        statuses: Statuses,
        round: GameRound,
        render_model: RenderModel,
    ) -> Self {
        Self {
            next_id: 0,
            time: Time::ZERO,
            units: Collection::new(),
            dead_units: Collection::new(),
            particles: Collection::new(),
            unit_templates,
            clan_effects,
            statuses,
            transition: false,
            round,
            config,
            vars: HashMap::new(),
            current_tick: TickModel::new(0),
            render_model,
        }
    }
}

impl TickModel {
    pub fn new(tick_num: usize) -> Self {
        Self {
            tick_time: Time::ZERO,
            tick_num,
            visual_timer: Time::new(UNIT_VISUAL_TIME),
        }
    }
}
