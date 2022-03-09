pub use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ChainDamageEffect {
    pub base_damage: Health,
    pub loss: R32,
    pub targets: usize,
    pub jump_distance: Coord,
}

impl Logic<'_> {
    pub fn process_chain_effect(
        &mut self,
        QueuedEffect {
            effect,
            caster,
            target,
        }: QueuedEffect<ChainDamageEffect>,
    ) {
        let mut touched = HashSet::new();
        let mut damage = effect.base_damage;
        let mut target = self.model.units.get(&target.unwrap()).unwrap();
        while touched.len() < effect.targets {
            touched.insert(target.id);
            self.effects.push(QueuedEffect {
                caster,
                target: Some(target.id),
                effect: Effect::Damage(DamageEffect {
                    hp: DamageValue::Absolute(damage),
                    lifesteal: DamageValue::Absolute(R32::ZERO),
                    on: default(),
                }),
            });
            damage -= damage * (effect.loss / r32(100.0));
            target = match self
                .model
                .units
                .iter()
                .filter(|unit| unit.faction == target.faction && !touched.contains(&unit.id))
                .filter(|unit| (unit.position - target.position).len() < effect.jump_distance)
                .min_by_key(|unit| (unit.position - target.position).len())
            {
                Some(unit) => unit,
                None => break,
            }
        }
    }
}
