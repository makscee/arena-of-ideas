use super::*;

impl Logic<'_> {
    pub fn check_condition(&self, condition: &Condition, context: &EffectContext) -> bool {
        match condition {
            Condition::Always => true,
            Condition::Not { condition } => !self.check_condition(&*condition, context),
            Condition::UnitHasStatus { who, status_type } => {
                let who = context.get(*who);
                let who = who
                    .and_then(|id| self.model.units.get(&id))
                    .expect("Caster, From, or Target not found");
                who.all_statuses
                    .iter()
                    .any(|status| status.status.name == *status_type)
            }
            Condition::UnitInjured { who } => {
                let who = context.get(*who);
                let who = who
                    .and_then(|id| self.model.units.get(&id))
                    .expect("Caster, From, or Target not found");
                who.stats.health < who.stats.max_hp
            }
            Condition::InRange { max_distance } => {
                let from = context
                    .from
                    .and_then(|id| self.model.units.get(&id))
                    .expect("Caster, From, or Target not found");
                let target = context
                    .target
                    .and_then(|id| self.model.units.get(&id))
                    .expect("Caster, From, or Target not found");
                (target.position - from.position).abs() <= *max_distance
            }
            Condition::Chance { percent } => {
                r32(global_rng().gen_range(0.0..=100.0)) < percent.calculate(&context, self)
            }
            Condition::Equal { a, b } => a.calculate(&context, self) == b.calculate(&context, self),
            Condition::Less { a, b } => a.calculate(&context, self) < b.calculate(&context, self),
            Condition::More { a, b } => a.calculate(&context, self) > b.calculate(&context, self),
            Condition::ClanSize { clan, count } => {
                self.model.config.clans.contains_key(clan)
                    && self.model.config.clans[clan] >= *count
            }
            Condition::HasClan { who, clan } => {
                let who = context
                    .get(*who)
                    .and_then(|id| self.model.units.get(&id))
                    .expect("Caster, From, or Target not found");
                who.clans.contains(clan)
            }
            Condition::HasVar { name } => context.vars.contains_key(name),
            Condition::Faction { who, faction } => {
                let who = context
                    .get(*who)
                    .and_then(|id| self.model.units.get(&id))
                    .expect("Caster, From, or Target not found");
                who.faction == *faction
            }
            Condition::And { a, b } => {
                self.check_condition(&*a, context) && self.check_condition(&*b, context)
            }
        }
    }
}
