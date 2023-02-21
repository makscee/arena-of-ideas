use super::*;

pub struct TimeSystem {
    pub game_time: Time,
}

impl TimeSystem {
    pub fn new() -> Self {
        Self { game_time: 0.0 }
    }
}

impl System for TimeSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        self.game_time += resources.delta_time;
        WorldSystem::set_time(self.game_time, world);
    }
}
