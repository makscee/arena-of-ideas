use super::*;

pub struct TimeSystem {}

impl TimeSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl System for TimeSystem {
    fn update(&mut self, _: &mut legion::World, resources: &mut Resources) {
        resources.global_time += resources.delta_time;
        GLOBAL_TIME.with(|map| {
            *map.borrow_mut() += resources.delta_time;
        });
    }
}
