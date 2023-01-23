use super::*;

mod shader_programs;

pub use shader_programs::*;

pub struct Assets {
    pub shader_programs: ShaderPrograms,
    pub camera: Camera2d,
}

impl Assets {
    pub fn new(geng: &Geng) -> Self {
        let shader_programs = ShaderPrograms::new(&geng);
        let camera = geng::Camera2d {
            center: vec2(0.0, 0.0),
            rotation: 0.0,
            fov: 5.0,
        };

        Self {
            shader_programs,
            camera,
        }
    }
}
