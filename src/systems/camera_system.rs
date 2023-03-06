use super::*;

pub struct CameraSystem {}

impl CameraSystem {
    pub fn new() -> Self {
        Self {}
    }
}

const CAMERA_FOLLOW_SPEED: f32 = 10.0;

impl System for CameraSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        let camera = &mut resources.camera;
        camera.pos += (camera.need_pos - camera.pos) * resources.delta_time * CAMERA_FOLLOW_SPEED;
        camera.camera.center = camera.pos;
    }
}

impl CameraSystem {}
