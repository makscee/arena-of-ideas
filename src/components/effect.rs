use super::*;

// enum inside component for API access
pub trait Effect {
    fn process(
        &self,
        context: &ContextComponent,
        resources: &mut Resources,
        world: &mut legion::World,
        effect_key: &PathBuf,
    ) -> Result<(), Error>;
}

// pub struct EffectComponent {
//     pub effect: Box<dyn Effect + Send + Sync>,
// }

// impl EffectComponent {
// pub fn process(
//     &self,
//     context: &mut ContextComponent,
//     resources: &mut Resources,
// ) -> Result<(), Error> {
//     self.effect.process(context, resources)
// }
// }
