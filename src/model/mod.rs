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
pub const SIDE_SLOTS: usize = 6;
pub const MAX_LIVES: i32 = 10;
pub const UNIT_TURN_TIME: f32 = 1.0;
pub const UNIT_PRE_TURN_TIME: f32 = 0.3;
pub const UNIT_SWITCH_TIME: f32 = 0.3;

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
    pub tick_num: Ticks,
}

#[derive(Clone)]
pub struct PhaseModel {
    pub enemy: Id,
    pub player: Id,
    pub turn_phase: TurnPhase,
}

#[derive(Clone)]
pub struct Model {
    pub next_id: Id,
    pub time: Time,
    pub units: Collection<Unit>,
    pub dead_units: Collection<Unit>,
    pub config: Config,
    pub round: usize,
    pub rounds: Vec<GameRound>,
    pub unit_templates: UnitTemplates,
    pub clan_effects: ClanEffects,
    pub statuses: Statuses,
    pub transition: bool,
    pub render_model: RenderModel,
    pub current_tick: TickModel,
    pub time_scale: f32,
    pub time_modifier: f32,
    pub lives: i32,
    /// Variables that persist for the whole game
    pub vars: HashMap<VarName, i32>,
    pub phase: PhaseModel,
}

impl Model {
    pub fn new(
        config: Config,
        unit_templates: UnitTemplates,
        clan_effects: ClanEffects,
        statuses: Statuses,
        round: usize,
        rounds: Vec<GameRound>,
        render_model: RenderModel,
        time_scale: f32,
        lives: i32,
    ) -> Self {
        Self {
            next_id: 0,
            time: Time::ZERO,
            units: Collection::new(),
            dead_units: Collection::new(),
            unit_templates,
            clan_effects,
            statuses,
            transition: false,
            round,
            rounds,
            config,
            vars: HashMap::new(),
            current_tick: TickModel::new(0),
            render_model,
            time_scale,
            time_modifier: time_scale,
            lives,
            phase: PhaseModel {
                enemy: 0,
                player: 0,
                turn_phase: TurnPhase::None,
            },
        }
    }

    pub fn get_who(&self, who: Who, context: &EffectContext) -> &Unit {
        let who_id = context.get_id(who);
        self.units
            .get(&who_id)
            .or(self.dead_units.get(&who_id))
            .expect(&format!("Can't find {}#{}", who, who_id))
    }

    pub fn get_who_mut(&mut self, who: Who, context: &EffectContext) -> &mut Unit {
        let who_id = context.get_id(who);
        self.units
            .get_mut(&who_id)
            .or(self.dead_units.get_mut(&who_id))
            .expect(&format!("Can't find {}#{}", who, who_id))
    }

    pub fn get(&self, id: Id, context: &EffectContext) -> &Unit {
        self.units
            .get(&id)
            .or(self.dead_units.get(&id))
            .expect(&format!("Can't find Unit#{}", id))
    }

    pub fn get_mut(&mut self, id: Id, context: &EffectContext) -> &mut Unit {
        self.units
            .get_mut(&id)
            .or(self.dead_units.get_mut(&id))
            .expect(&format!("Can't find Unit#{}", id))
    }
}

impl TickModel {
    pub fn new(tick_num: Ticks) -> Self {
        Self {
            tick_time: Time::ZERO,
            tick_num,
        }
    }
}
