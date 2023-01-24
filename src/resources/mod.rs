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
        let shader_programs = ShaderPrograms::new();
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

    pub fn load(&mut self, fws: &mut FileWatcherSystem) {
        self.load_shader_programs(fws);
    }

    fn load_shader_programs(&mut self, fws: &mut FileWatcherSystem) {
        let list = futures::executor::block_on(ShaderPrograms::load_shaders_list(&self.geng));
        for file in list {
            fws.load_and_watch_file(
                self,
                &static_path().join(file),
                Box::new(Self::load_shader_program),
            );
        }
    }

    fn load_shader_program(resources: &mut Resources, file: &PathBuf) {
        let program = futures::executor::block_on(<ugli::Program as geng::LoadAsset>::load(
            &resources.geng,
            &static_path().join(file),
        ))
        .expect(&format!("Failed to load shader {:?}", file));
        debug!("Load shader program {:?}", file);
        resources
            .shader_programs
            .insert_program(file.clone(), program)
    }
}
