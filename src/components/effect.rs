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