use super::*;

pub struct VisualQueueSystem {}

impl VisualQueueSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl System for VisualQueueSystem {
    fn update(&mut self, _world: &mut legion::World, resources: &mut Resources) {
        resources.visual_queue.current_ts += resources.delta_time;
    }

    fn draw(
        &self,
        _world: &legion::World,
        _resources: &Resources,
        _framebuffer: &mut ugli::Framebuffer,
    ) {
    }
}
