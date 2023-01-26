use super::*;

pub struct AttackComponent {
    pub value: Hp,
}

#[derive(Debug)]
pub enum AttackEffect {
    DealDamage { value: Option<Hp> },
}

impl Effect for AttackEffect {
    fn process(
        &self,
        context: &ContextComponent,
        resources: &mut Resources,
        world: &mut legion::World,
        effect_key: &EffectKey,
    ) -> Result<(), Error> {
        match self {
            AttackEffect::DealDamage { value } => {
                let value = match value {
                    Some(value) => *value,
                    None => {
                        world
                            .entry(context.creator)
                            .context("Can't find creator")?
                            .get_component::<AttackComponent>()?
                            .value
                    }
                };

                let next_effect_key = effect_key.join(self.to_string());
                resources.effects_storage.insert(
                    next_effect_key.clone(),
                    Box::new(HpEffect::TakeDamage { value }),
                );

                resources.action_queue.push_front(Action {
                    context: context.clone(),
                    effect_key: next_effect_key,
                });
                Ok(())
            }
        }
    }
}

impl fmt::Display for AttackEffect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}
