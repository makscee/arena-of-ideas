use super::*;

pub struct ActionSystem {}

impl ActionSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl System for ActionSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        let Some(action) = resources.action_queue.pop_front() else { return };
        let effect = resources
            .effects_storage
            .remove(&action.effect_key)
            .expect("Effect not loaded");
        match effect.process(&action.context, resources, world, &action.effect_key) {
            Ok(_) => {}
            Err(error) => error!("Effect process error: {}", error),
        }
        resources.effects_storage.insert(action.effect_key, effect);
    }

    fn draw(
        &self,
        world: &legion::World,
        resources: &Resources,
        framebuffer: &mut ugli::Framebuffer,
    ) {
    }
}

pub struct Action {
    pub context: ContextComponent,
    pub effect_key: PathBuf,
}
