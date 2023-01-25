use super::*;

pub struct TimeSystem {
    pub game_time: Time,
    pub simulation_time: Time,
}

impl TimeSystem {
    pub fn new() -> Self {
        Self {
            game_time: 0.0,
            simulation_time: 0.0,
        }
    }
}

impl System for TimeSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        self.game_time += resources.delta_time;
    }

    fn draw(
        &self,
        world: &legion::World,
        resources: &Resources,
        framebuffer: &mut ugli::Framebuffer,
    ) {
    }
}
