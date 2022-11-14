use super::*;

impl Logic {
    pub fn check_condition(model: &Model, condition: &Condition, context: &EffectContext) -> bool {
        match condition {
            Condition::Always => true,
            Condition::Not { condition } => !Self::check_condition(model, &*condition, context),
            Condition::UnitHasStatus { who, status_type } => {
                let who = model.get_who(*who, &context);
                who.all_statuses
                    .iter()
                    .any(|status| status.status.name == *status_type)
            }
            Condition::InRange { max_distance } => {
                let owner = model.get_who(Who::Owner, &context);
                let target = model.get_who(Who::Target, &context);
                distance_between_units(owner, target) <= *max_distance
            }
            Condition::Chance { percent } => {
                global_rng().gen_range(0..=100) < percent.calculate(&context, model)
            }
            Condition::Equal { a, b } => {
                a.calculate(&context, model) == b.calculate(&context, model)
            }
            Condition::Less { a, b } => a.calculate(&context, model) < b.calculate(&context, model),
            Condition::More { a, b } => a.calculate(&context, model) > b.calculate(&context, model),
            Condition::ClanSize { clan, count } => {
                model.config.clans.contains_key(clan) && model.config.clans[clan] >= *count
            }
            Condition::HasClan { who, clan } => {
                let who = model.get_who(*who, &context);
                who.clans.contains(clan)
            }
            Condition::HasVar { name } => context.vars.contains_key(name),
            Condition::Faction { who, faction } => {
                let who = model.get_who(*who, &context);
                who.faction == *faction
            }
            Condition::And { a, b } => {
                Self::check_condition(model, &*a, context)
                    && Self::check_condition(model, &*b, context)
            }
            Condition::Position { who, position } => {
                let who = model.get_who(*who, &context);
                who.position.x == *position
            }
        }
    }
}
