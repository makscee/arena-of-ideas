use super::*;

pub struct CassettePlayerSystem {
    current_ts: Time,
}

impl CassettePlayerSystem {
    pub fn new() -> Self {
        Self { current_ts: 0.0 }
    }
}

impl System for CassettePlayerSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        self.current_ts += resources.delta_time;
        self.current_ts = self.current_ts.min(resources.cassette.length());
        resources.cassette.current_ts = self.current_ts;
    }

    fn draw(
        &self,
        world: &legion::World,
        resources: &Resources,
        framebuffer: &mut ugli::Framebuffer,
    ) {
    }
}
