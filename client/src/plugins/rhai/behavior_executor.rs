use super::*;
use schema::{Target, Tier};

pub trait TargetResolver {
    fn resolve_targets(&self, ctx: &mut ClientContext) -> NodeResult<Vec<u64>>;
}

impl TargetResolver for Target {
    fn resolve_targets(&self, ctx: &mut ClientContext) -> NodeResult<Vec<u64>> {
        use rand::seq::IndexedRandom;

        let owner_id = ctx.owner()?;
        let battle = ctx.battle_mut()?;

        match self {
            Target::Owner => Ok(vec![owner_id]),
            Target::RandomEnemy => {
                let enemies = battle.all_enemies(owner_id)?.clone();
                Ok(enemies
                    .choose(&mut battle.rng)
                    .copied()
                    .into_iter()
                    .collect_vec())
            }
            Target::AllEnemies => battle.all_enemies(owner_id).cloned(),
            Target::RandomAlly => {
                let allies = battle.all_allies(owner_id)?.clone();
                Ok(allies
                    .choose(&mut battle.rng)
                    .copied()
                    .into_iter()
                    .collect_vec())
            }
            Target::AllAllies => battle.all_allies(owner_id).cloned(),
            Target::All => Ok(battle
                .all_allies(owner_id)?
                .into_iter()
                .chain(battle.all_enemies(owner_id)?.into_iter())
                .copied()
                .collect_vec()),
            Target::Caster => ctx.caster().to_not_found().map(|id| vec![id]),
            Target::Attacker => ctx.attacker().to_not_found().map(|id| vec![id]),
            Target::Target => ctx.target().to_not_found().map(|id| vec![id]),
            Target::AdjacentBack => battle
                .offset_unit(owner_id, -1)
                .to_not_found()
                .map(|id| vec![id]),
            Target::AdjacentFront => battle
                .offset_unit(owner_id, 1)
                .to_not_found()
                .map(|id| vec![id]),
            Target::AllyAtSlot(slot) => {
                let allies = battle.all_allies(owner_id)?;
                Ok(allies
                    .get(*slot as usize)
                    .copied()
                    .into_iter()
                    .collect_vec())
            }
            Target::EnemyAtSlot(slot) => {
                let enemies = battle.all_enemies(owner_id)?;
                Ok(enemies
                    .get(*slot as usize)
                    .copied()
                    .into_iter()
                    .collect_vec())
            }
            Target::List(targets) => {
                let mut all = Vec::new();
                for target in targets {
                    all.extend(target.resolve_targets(ctx)?);
                }
                Ok(all)
            }
        }
    }
}

impl Tier for NUnitBehavior {
    fn tier(&self) -> u8 {
        let trigger_tier = self.trigger.tier();
        let target_tier = self.target.tier();
        let effect_tier = self.effect.tier();
        (trigger_tier + target_tier + effect_tier) / 3
    }
}

impl Tier for NStatusBehavior {
    fn tier(&self) -> u8 {
        let trigger_tier = self.trigger.tier();
        let effect_tier = self.effect.tier();
        (trigger_tier + effect_tier) / 2
    }
}

impl Tier for NAbilityEffect {
    fn tier(&self) -> u8 {
        self.effect.tier()
    }
}
