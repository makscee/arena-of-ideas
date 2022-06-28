use std::{path::PathBuf, sync::mpsc::Receiver};

use geng::prelude::*;
use notify::{DebouncedEvent, RecommendedWatcher, Watcher};

use crate::model::ShaderParameters;

#[derive(clap::Args)]
pub struct ShaderEdit {
    config_path: String,
}

#[derive(Deserialize, geng::Assets)]
#[asset(json)]
#[serde(deny_unknown_fields)]
struct ShaderEditConfig {
    path: PathBuf,
    parameters: ShaderParameters,
}

impl ShaderEdit {
    pub fn run(self, geng: &Geng) -> Box<dyn geng::State> {
        Box::new(EditState::new(geng, self.config_path))
    }
}

type Time = R32;

struct EditState {
    geng: Geng,
    time: Time,
    config: ShaderEditConfig,
    loaded: Option<ugli::Program>,
    receiver: Receiver<DebouncedEvent>,
    watcher: RecommendedWatcher,
}

impl EditState {
    pub fn new(geng: &Geng, config_path: String) -> Self {
        // Load config
        let config_path = static_path().join(&config_path);
        let mut config = futures::executor::block_on(<ShaderEditConfig as geng::LoadAsset>::load(
            geng,
            &config_path,
        ))
        .unwrap();
        config.path = static_path().join(&config.path);

        // Setup watcher
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher: RecommendedWatcher =
            notify::Watcher::new(tx, std::time::Duration::from_secs(1))
                .expect("Failed to initialize a watcher");
        watcher
            .watch(&config_path, notify::RecursiveMode::NonRecursive)
            .expect(&format!("Failed to start watching {config_path:?}"));
        watcher
            .watch(&config.path, notify::RecursiveMode::NonRecursive)
            .expect(&format!("Failed to start watching {:?}", config.path));

        Self {
            geng: geng.clone(),
            time: Time::ZERO,
            loaded: None,
            config,
            watcher,
            receiver: rx,
        }
    }
}

impl geng::State for EditState {
    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);
        self.time += delta_time;

        use std::sync::mpsc::TryRecvError;
        match self.receiver.try_recv() {
            Ok(event) => debug!("{event:?}"),
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) => {
                error!("Disconnected from the channel");
            }
        }
    }

    fn draw(&mut self, framebuffer: &mut ugli::Framebuffer) {
        ugli::clear(framebuffer, Some(Color::BLACK), None);

        let camera = geng::Camera2d {
            center: vec2(0.0, 0.0),
            rotation: 0.0,
            fov: 22.0,
        };

        if let Some(program) = &self.loaded {
            let quad = ugli::VertexBuffer::new_dynamic(
                self.geng.ugli(),
                vec![
                    draw_2d::Vertex {
                        a_pos: vec2(-1.0, -1.0),
                    },
                    draw_2d::Vertex {
                        a_pos: vec2(1.0, -1.0),
                    },
                    draw_2d::Vertex {
                        a_pos: vec2(1.0, 1.0),
                    },
                    draw_2d::Vertex {
                        a_pos: vec2(-1.0, 1.0),
                    },
                ],
            );
            let uniforms = (
                ugli::uniforms! {
                    u_time: self.time.as_f32(),
                },
                geng::camera2d_uniforms(&camera, framebuffer.size().map(|x| x as f32)),
                &self.config.parameters,
            );
            ugli::draw(
                framebuffer,
                program,
                ugli::DrawMode::TriangleFan,
                &quad,
                &uniforms,
                ugli::DrawParameters::default(),
            );
        }
    }
}
