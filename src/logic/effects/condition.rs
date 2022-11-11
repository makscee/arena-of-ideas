use super::*;

impl Logic {
    pub fn check_condition(&self, condition: &Condition, context: &EffectContext) -> bool {
        match condition {
            Condition::Always => true,
            Condition::Not { condition } => !self.check_condition(&*condition, context),
            Condition::UnitHasStatus { who, status_type } => {
                let who = self.model.get(*who, &context);
                who.all_statuses
                    .iter()
                    .any(|status| status.status.name == *status_type)
            }
            Condition::InRange { max_distance } => {
                let owner = self.model.get(Who::Owner, &context);
                let target = self.model.get(Who::Target, &context);
                distance_between_units(owner, target) <= *max_distance
            }
            Condition::Chance { percent } => {
                global_rng().gen_range(0..=100) < percent.calculate(&context, self)
            }
            Condition::Equal { a, b } => a.calculate(&context, self) == b.calculate(&context, self),
            Condition::Less { a, b } => a.calculate(&context, self) < b.calculate(&context, self),
            Condition::More { a, b } => a.calculate(&context, self) > b.calculate(&context, self),
            Condition::ClanSize { clan, count } => {
                self.model.config.clans.contains_key(clan)
                    && self.model.config.clans[clan] >= *count
            }
            Condition::HasClan { who, clan } => {
                let who = self.model.get(*who, &context);
                who.clans.contains(clan)
            }
            Condition::HasVar { name } => context.vars.contains_key(name),
            Condition::Faction { who, faction } => {
                let who = self.model.get(*who, &context);
                who.faction == *faction
            }
            Condition::And { a, b } => {
                self.check_condition(&*a, context) && self.check_condition(&*b, context)
            }
            Condition::Position { who, position } => {
                let who = self.model.get(*who, &context);
                who.position.x == *position
            }
        }
    }
}
