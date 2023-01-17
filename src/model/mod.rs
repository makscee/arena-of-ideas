use super::*;
mod condition;
mod consts;
mod expr;
mod unit;

pub use condition::Condition;
pub use consts::*;
pub use expr::*;
use geng::prelude::itertools::Itertools;
pub use unit::*;

pub struct Model {
    pub units: Collection<Unit>,
    pub game_time: Time,
    pub vars: HashMap<VarName, i32>,
}

impl Model {
    pub fn new() -> Self {
        Self {
            units: default(),
            game_time: 0.0,
            vars: default(),
        }
    }
    pub fn get_who(&self, who: Who, context: &LogicEffectContext) -> &Unit {
        let who_id = context.get_id(who);
        self.units
            .get(&who_id)
            .expect(&format!("Can't find {}#{}", who, who_id))
    }

    pub fn get_who_mut(&mut self, who: Who, context: &LogicEffectContext) -> &mut Unit {
        let who_id = context.get_id(who);
        self.units
            .get_mut(&who_id)
            .expect(&format!("Can't find {}#{}", who, who_id))
    }

    pub fn get(&self, id: Id) -> &Unit {
        self.units
            .get(&id)
            .expect(&format!("Can't find Unit#{}", id))
    }

    pub fn get_mut(&mut self, id: Id) -> &mut Unit {
        self.units
            .get_mut(&id)
            .expect(&format!("Can't find Unit#{}", id))
    }

    pub fn get_all(&self, context: &LogicEffectContext) -> Vec<&Unit> {
        let mut result: Vec<&Unit> = vec![];
        self.units.iter().for_each(|unit| {
            if unit.id == context.creator || unit.id == context.owner || unit.id == context.target {
                result.push(unit);
            }
        });
        result
    }

    pub fn check_condition(&self, condition: &Condition, context: &LogicEffectContext) -> bool {
        match condition {
            Condition::Always => true,
            Condition::Not { condition } => !self.check_condition(condition, context),
            // Condition::UnitHasStatus { who, status_type } => {
            //     let who = self.get_who(*who, &context);
            //     who.all_statuses
            //         .iter()
            //         .any(|status| status.status.name == *status_type)
            // }
            Condition::Chance { percent } => {
                global_rng().gen_range(0..=100) < percent.calculate(&context, self)
            }
            Condition::Equal { a, b } => a.calculate(&context, self) == b.calculate(&context, self),
            Condition::Less { a, b } => a.calculate(&context, self) < b.calculate(&context, self),
            Condition::More { a, b } => a.calculate(&context, self) > b.calculate(&context, self),
            Condition::HasClan { who, clan } => {
                let who = self.get_who(*who, &context);
                who.clans.iter().contains(clan)
            }
            Condition::HasVar { name } => context.vars.contains_key(name),
            Condition::Faction { who, faction } => {
                let who = self.get_who(*who, &context);
                who.faction == *faction
            }
            Condition::And { conditions } => conditions
                .iter()
                .all(|condition| Self::check_condition(self, condition, context)),
            Condition::Or { conditions } => conditions
                .iter()
                .any(|condition| Self::check_condition(self, condition, context)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Who {
    Owner,
    Creator,
    Target,
}

impl fmt::Display for Who {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

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
