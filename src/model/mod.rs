use super::*;
mod condition;
mod consts;
mod expr;
mod shader_config;
mod status;
mod unit;

use condition::Condition;
pub use condition::*;
pub use consts::*;
pub use expr::*;
pub use shader_config::*;
pub use status::*;
pub use unit::*;

pub struct Model {
    pub battle_units: Collection<Unit>,
    pub game_time: Time,
}

impl Model {
    pub fn new() -> Self {
        Self {
            battle_units: default(),
            game_time: 0.0,
        }
    }
}

impl Model {
    pub fn get_who(&self, who: Who, context: &EffectContext) -> &Unit {
        let who_id = context.get_id(who);
        self.units
            .get(&who_id)
            .expect(&format!("Can't find {}#{}", who, who_id))
    }

    pub fn get_who_mut(&mut self, who: Who, context: &EffectContext) -> &mut Unit {
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

    pub fn get_all(&self, context: &EffectContext) -> Vec<&Unit> {
        let mut result: Vec<&Unit> = vec![];
        self.units
            .iter()
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
                owner.position.distance(&target.position) <= *max_distance
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
            Condition::And { conditions } => conditions
                .iter()
                .all(|condition| Self::check_condition(self, condition, context)),
            Condition::Position { who, position } => {
                let who = self.get_who(*who, &context);
                who.position.x == *position
            }
            Condition::Or { conditions } => conditions
                .iter()
                .any(|condition| Self::check_condition(self, condition, context)),
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
