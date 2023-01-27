use legion::EntityStore;

use super::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Effect {
    DealDamage { value: Hp },
    Repeat { count: usize, effect: Box<Effect> },
}

impl Effect {
    pub fn process(
        &self,
        context: &Context,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> Result<(), Error> {
        match self {
            Effect::DealDamage { value } => {
                world
                    .entry(context.target)
                    .context("Failed to get Target")?
                    .get_component_mut::<HpComponent>()?
                    .current -= value;
            }
            Effect::Repeat { count, effect } => {
                for _ in 0..*count {
                    resources
                        .action_queue
                        .push_front(Action::new(context.clone(), effect.deref().clone()));
                }
            }
        }
        Ok(())
    }
}
