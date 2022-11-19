use std::collections::VecDeque;

use super::*;

mod ability;
mod clans;
mod condition;
mod effect;
mod expr;
mod factions;
mod modifier;
mod panel;
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
pub use panel::*;
pub use particle::*;
pub use position::*;
pub use render::*;
pub use status::*;
pub use unit::*;

// TODO: make configurable
pub const SIDE_SLOTS: usize = 5;
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
    pub team: Vec<Unit>,
    pub units: Collection<Unit>,
    pub dead_units: Collection<Unit>,
    pub config: Config,
    pub round: usize,
    pub rounds: Vec<GameRound>,
    pub unit_templates: UnitTemplates,
    pub clan_effects: ClanEffects,
    pub statuses: Statuses,
    pub transition: bool,
    pub in_battle: bool,
    pub render_model: RenderModel,
    pub time_scale: f32,
    pub time_modifier: f32,
    pub lives: i32,
    /// Variables that persist for the whole game
    pub vars: HashMap<VarName, i32>,
    pub phase: PhaseModel,
    pub shop: Shop,
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
        shop: Shop,
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
            in_battle: false,
            round,
            rounds,
            config,
            vars: HashMap::new(),
            render_model,
            time_scale,
            time_modifier: time_scale,
            lives,
            team: vec![],
            phase: PhaseModel {
                enemy: 0,
                player: 0,
                turn_phase: TurnPhase::None,
            },
            shop,
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

    pub fn get_all(&self, context: &EffectContext) -> Vec<&Unit> {
        let mut result: Vec<&Unit> = vec![];
        self.units
            .iter()
            .chain(self.dead_units.iter())
            .for_each(|unit| {
                if unit.id == context.creator
                    || unit.id == context.owner
                    || unit.id == context.target
                {
                    result.push(unit);
                }
            });
        result
    }

    pub fn check_condition(&self, condition: &Condition, context: &EffectContext) -> bool {
        match condition {
            Condition::Always => true,
            Condition::Not { condition } => !self.check_condition(condition, context),
            Condition::UnitHasStatus { who, status_type } => {
                let who = self.get_who(*who, &context);
                who.all_statuses
                    .iter()
                    .any(|status| status.status.name == *status_type)
            }
            Condition::InRange { max_distance } => {
                let owner = self.get_who(Who::Owner, &context);
                let target = self.get_who(Who::Target, &context);
                distance_between_units(owner, target) <= *max_distance
            }
            Condition::Chance { percent } => {
                global_rng().gen_range(0..=100) < percent.calculate(&context, self)
            }
            Condition::Equal { a, b } => a.calculate(&context, self) == b.calculate(&context, self),
            Condition::Less { a, b } => a.calculate(&context, self) < b.calculate(&context, self),
            Condition::More { a, b } => a.calculate(&context, self) > b.calculate(&context, self),
            Condition::ClanSize { clan, count } => {
                self.config.clans.contains_key(clan) && self.config.clans[clan] >= *count
            }
            Condition::HasClan { who, clan } => {
                let who = self.get_who(*who, &context);
                who.clans.contains(clan)
            }
            Condition::HasVar { name } => context.vars.contains_key(name),
            Condition::Faction { who, faction } => {
                let who = self.get_who(*who, &context);
                who.faction == *faction
            }
            Condition::And { a, b } => {
                Self::check_condition(self, &*a, context)
                    && Self::check_condition(self, &*b, context)
            }
            Condition::Position { who, position } => {
                let who = self.get_who(*who, &context);
                who.position.x == *position
            }
            Condition::Or { a, b } => {
                Self::check_condition(self, &*a, context)
                    || Self::check_condition(self, &*b, context)
            }
        }
    }

    pub fn calculate_clan_members<'a>(&mut self) {
        let unique_units = self
            .team
            .iter()
            .map(|unit| (&unit.unit_type, &unit.clans))
            .collect::<HashMap<_, _>>();
        let mut clans = HashMap::new();
        for clan in unique_units.into_values().flatten() {
            *clans.entry(*clan).or_insert(0) += 1;
        }
        self.config.clans = clans;
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
