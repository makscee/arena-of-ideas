use geng::prelude::anyhow::{Error, Ok};

use super::*;

pub type Hp = i32;
pub struct HpComponent {
    pub current: Hp,
    pub max: Hp,
}

#[derive(Debug)]
pub enum HpEffect {
    TakeDamage { value: Hp },
    // TakeHeal { value: Hp },
    // ChangeMax { delta: Hp },
}

impl fmt::Display for HpEffect {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Effect for HpEffect {
    fn process(
        &self,
        context: &ContextComponent,
        _resources: &mut Resources,
        world: &mut legion::World,
        _effect_key: &PathBuf,
    ) -> Result<(), Error> {
        match self {
            HpEffect::TakeDamage { value } => {
                world
                    .entry(context.target)
                    .context("Can't find target")?
                    .get_component_mut::<HpComponent>()?
                    .current -= value;
                debug!("take dmg {}", value);
                // resources.action_queue.push_front(Action {
                //     context: context.clone(),
                //     effect_key,
                // });
                Ok(())
            } // HpEffect::TakeHeal { value } => {
              //     world
              //         .entry(context.target)
              //         .context("Target not found")?
              //         .get_component_mut::<HpComponent>()?
              //         .current += value;
              //     Ok(())
              // }
              // HpEffect::ChangeMax { delta } => {
              //     world
              //         .entry(context.target)
              //         .context("Target not found")?
              //         .get_component_mut::<HpComponent>()?
              //         .max += delta;
              //     Ok(())
              // }
        }
    }
}
