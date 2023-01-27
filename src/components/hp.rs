use geng::prelude::anyhow::{Error, Ok};

use super::*;

pub type Hp = i32;
pub struct HpComponent {
    pub current: Hp,
    pub max: Hp,
}

#[derive(Debug, Deserialize)]
pub enum HpEffect {
    TakeDamage { value: Hp },
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
                Ok(())
            }
        }
    }
}
