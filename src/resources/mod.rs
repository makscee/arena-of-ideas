use super::*;

mod shader_programs;

pub use shader_programs::*;

pub struct Resources {
    pub shader_programs: ShaderPrograms,
    pub down_key: Option<Key>,

    pub game_time: Time,
    pub delta_time: Time,

    pub camera: Camera2d,
    pub geng: Geng,
}

impl Resources {
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
            down_key: default(),
            geng: geng.clone(),
            game_time: default(),
            delta_time: default(),
        }
    }
}
