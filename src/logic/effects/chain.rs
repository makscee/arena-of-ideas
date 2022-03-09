pub use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ChainEffect {
    pub targets: usize,
    pub jump_distance: Coord,
    pub effect: Effect,
    pub jump_modifier: Modifier,
}

impl Logic<'_> {
    pub fn process_chain_effect(
        &mut self,
        QueuedEffect {
            effect,
            caster,
            target,
        }: QueuedEffect<ChainEffect>,
    ) {
        let mut touched = HashSet::new();
        let mut touch_effect = effect.effect.clone();
        let mut target = self.model.units.get(&target.unwrap()).unwrap();
        while touched.len() < effect.targets {
            touched.insert(target.id);
            self.effects.push(QueuedEffect {
                caster,
                target: Some(target.id),
                effect: touch_effect.clone(),
            });
            touch_effect.apply_modifier(&effect.jump_modifier);
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
