use super::*;

pub struct Camera {
    pub camera: geng::Camera2d,
    pub pos: vec2<f32>,
    pub focus: Focus,
}

pub enum Focus {
    Shop,
    Battle,
}

impl Camera {
    pub fn new(options: &Options) -> Self {
        let camera = geng::Camera2d {
            center: vec2(0.0, 0.0),
            rotation: 0.0,
            fov: options.fov,
        };
        let pos = vec2::ZERO;
        Self {
            camera,
            pos,
            focus: Focus::Shop,
        }
    }
}
