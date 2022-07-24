use super::*;

mod condition;
mod modifiers;

#[derive(Clone)]
pub struct QueuedEffect<T> {
    pub effect: T,
    pub context: EffectContext,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EffectContext {
    pub caster: Option<Id>,
    pub from: Option<Id>,
    pub target: Option<Id>,
    pub vars: HashMap<VarName, R32>,
    pub status_id: Option<Id>,
}

impl EffectContext {
    pub fn get(&self, who: Who) -> Option<Id> {
        match who {
            Who::Caster => self.caster,
            Who::From => self.from,
            Who::Target => self.target,
        }
    }
    pub fn to_string(&self, logic: &Logic) -> String {
        format!(
            "caster: {}, from: {}, target: {}",
            self.unit_to_string(self.caster, logic),
            self.unit_to_string(self.from, logic),
            self.unit_to_string(self.target, logic),
        )
    }
    pub fn unit_to_string(&self, unit: Option<Id>, logic: &Logic) -> String {
        match unit {
            Some(id) => {
                if let Some(unit) = logic.model.units.get(&id) {
                    format!("{}#{}", unit.unit_type, id)
                } else {
                    let unit = logic.model.dead_units.get(&id).unwrap();
                    format!("{}#{}(dead)", unit.unit_type, id)
                }
            }
            None => "None".to_owned(),
        }
    }
}

impl Logic {
    pub fn process_effects(&mut self) {
        const MAX_ITERATIONS: usize = 1000;
        let mut iterations = 0;
        while let Some(QueuedEffect {
            effect,
            mut context,
        }) = self.effects.pop_front()
        {
            self.model.vars.iter().for_each(|v| {
                if !context.vars.contains_key(v.0) {
                    context.vars.insert(v.0.clone(), *v.1);
                }
            });
            trace!("Processing {:?} on {}", effect, context.to_string(self));
            effect.as_box().process(context, self);

            iterations += 1;
            if iterations > MAX_ITERATIONS {
                error!("Exceeded effect processing limit: {}", MAX_ITERATIONS);
                break;
            }
        }
    }
}
