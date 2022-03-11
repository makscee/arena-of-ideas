pub use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct AddTargetsEffect {
    pub additional_targets: Option<usize>,
    pub effect: Effect,
}

impl AddTargetsEffect {
    pub fn walk_children_mut(&mut self, f: &mut impl FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl Logic<'_> {
    pub fn process_add_targets_effect(
        &mut self,
        QueuedEffect {
            effect,
            caster,
            target,
        }: QueuedEffect<AddTargetsEffect>,
    ) {
        let caster = caster
            .and_then(|id| self.model.units.get(&id))
            .expect("Caster not found");
        let target = target
            .and_then(|id| self.model.units.get(&id))
            .expect("Target not found");
        let mut targets: HashSet<Id> = default();
        targets.insert(target.id);
        while match effect.additional_targets {
            Some(num) => targets.len() < 1 + num,
            None => true,
        } {
            if let Some(another) = self
                .model
                .units
                .iter()
                .filter(|unit| unit.faction == target.faction)
                .filter(|unit| !targets.contains(&unit.id))
                .filter(|unit| distance_between_units(unit, caster) < caster.attack.radius)
                .choose(&mut global_rng())
            {
                targets.insert(another.id);
            } else {
                break;
            }
        }
        for target in targets {
            self.effects.push(QueuedEffect {
                effect: effect.effect.clone(),
                caster: Some(caster.id),
                target: Some(target),
            });
        }
    }
}
