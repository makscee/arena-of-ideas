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
    #[serde(default)]
    extra_watch: Vec<PathBuf>,
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
    config_path: PathBuf,
    config: ShaderEditConfig,
    shader: Option<(PathBuf, ugli::Program)>,
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
        .expect("Failed to load config");
        config.path = static_path().join(&config.path);
        config.extra_watch = config
            .extra_watch
            .iter()
            .map(|path| static_path().join(&path))
            .collect();

        // Load shader
        let program = futures::executor::block_on(<ugli::Program as geng::LoadAsset>::load(
            geng,
            &config.path,
        ))
        .expect("Failed to load shader");

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
        config.extra_watch.iter().for_each(|path| {
            watcher
                .watch(&path, notify::RecursiveMode::NonRecursive)
                .expect(&format!("Failed to start watching {:?}", path))
        });

        Self {
            geng: geng.clone(),
            time: Time::ZERO,
            shader: Some((config.path.clone(), program)),
            config_path,
            config,
            watcher,
            receiver: rx,
        }
    }

    fn handle_notify(&mut self, event: notify::DebouncedEvent) {
        debug!("Notify event: {event:?}");
        match event {
            DebouncedEvent::NoticeWrite(path)
            | DebouncedEvent::Create(path)
            | DebouncedEvent::Write(path) => self.reload_path(path),
            DebouncedEvent::NoticeRemove(path) => {
                // (Neo)vim writes the file by removing and recreating it,
                // hence this hack
                self.switch_watch(&path, &path);
                self.reload_path(path);
            }
            DebouncedEvent::Remove(_) => todo!(),
            DebouncedEvent::Error(error, path) => {
                error!("Notify error on path {path:?}: {error}");
            }
            _ => {}
        }
    }

    fn reload_path(&mut self, path: PathBuf) {
        if path == self.config.path {
            self.reload_shader();
        } else if path == self.config_path {
            self.config = ShaderEditConfig::load(&self.geng, path).expect("Failed to load config");
            self.reload_shader();
        } else if self.config.extra_watch.contains(&path) {
            self.reload_shader();
        } else {
            warn!("Tried to reload an unregistered path (neither config nor shader): {path:?}");
        }
    }

    fn switch_watch(
        &mut self,
        old_path: impl AsRef<std::path::Path>,
        new_path: impl AsRef<std::path::Path>,
    ) {
        if let Err(error) = self.watcher.unwatch(old_path.as_ref()) {
            error!(
                "Failed to unwatch old shader path ({:?}): {error}",
                old_path.as_ref()
            );
        }
        if let Err(error) = self
            .watcher
            .watch(new_path.as_ref(), notify::RecursiveMode::NonRecursive)
        {
            error!(
                "Failed to start watching shader on {:?}: {error}",
                new_path.as_ref()
            );
        }
    }

    fn reload_shader(&mut self) {
        // Stop watching old shader if the path has changed
        if let Some(path) = self.shader.as_ref().map(|(path, _)| path.clone()) {
            if path != self.config.path {
                self.switch_watch(path, self.config.path.clone());
            }
        }

        // Reload shader
        let program = match futures::executor::block_on(<ugli::Program as geng::LoadAsset>::load(
            &self.geng,
            &self.config.path,
        )) {
            Ok(program) => program,
            Err(error) => {
                error!("Failed to load program: {error}");
                return;
            }
        };
        self.shader = Some((self.config.path.clone(), program));
    }
}

impl ShaderEditConfig {
    fn load(geng: &Geng, full_path: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let mut config = futures::executor::block_on(<ShaderEditConfig as geng::LoadAsset>::load(
            geng,
            full_path.as_ref(),
        ))?;
        config.path = static_path().join(&config.path);
        Ok(config)
    }
}

impl geng::State for EditState {
    fn update(&mut self, delta_time: f64) {
        let delta_time = Time::new(delta_time as _);
        self.time += delta_time;

        use std::sync::mpsc::TryRecvError;
        match self.receiver.try_recv() {
            Ok(event) => self.handle_notify(event),
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
            fov: 15.0,
        };

        if let Some((_, program)) = &self.shader {
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
                    u_unit_position: Vec2::<f32>::ZERO,
                    u_unit_radius: 1_f32,
                    u_window_size: self.geng.window().size(),
                    u_spawn: self.time.as_f32().fract(),
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
                ugli::DrawParameters {
                    blend_mode: Some(default()),
                    ..default()
                },
            );
        }
    }
}
