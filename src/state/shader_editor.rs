use crate::assets::load_shader_library;

use super::*;

use geng::prelude::*;
use geng::ui::*;
use std::{path::PathBuf, sync::mpsc::Receiver};

use notify::{DebouncedEvent, RecommendedWatcher, Watcher};

type Time = R32;

pub struct ShaderEditState {
    geng: Geng,
    assets: Rc<Assets>,
    view: Rc<View>,
    time: Time,
    shader: ShaderProgram,
    receiver: Receiver<DebouncedEvent>,
    watcher: RecommendedWatcher,
    watched_list: Vec<PathBuf>,
}

impl ShaderEditState {
    pub fn new(geng: &Geng, assets: Rc<Assets>, view: Rc<View>) -> Self {
        // Setup watcher
        let (tx, rx) = std::sync::mpsc::channel();
        let watcher: RecommendedWatcher =
            notify::Watcher::new(tx, std::time::Duration::from_secs(1))
                .expect("Failed to initialize a watcher");

        let mut state = Self {
            geng: geng.clone(),
            time: Time::ZERO,
            shader: assets.system_shaders.unit.clone(),
            watched_list: default(),
            watcher,
            assets,
            view,
            receiver: rx,
        };
        state.reload_list();
        state.watch_all();
        state
    }

    fn handle_notify(&mut self, event: notify::DebouncedEvent) {
        debug!("Notify event: {event:?}");
        match event {
            DebouncedEvent::NoticeWrite(path)
            // | DebouncedEvent::Write(path)
            | DebouncedEvent::Create(path) => self.reload_watch(),
            DebouncedEvent::NoticeRemove(path) => {
                // (Neo)vim writes the file by removing and recreating it,
                // hence this hack
                // self.switch_watch(&path, &path);
                // self.reload_path(path);

                self.reload_watch();
            }
            DebouncedEvent::Remove(_) => todo!(),
            DebouncedEvent::Error(error, path) => {
                error!("Notify error on path {path:?}: {error}");
            }
            _ => {}
        }
    }

    fn unwatch_all(&mut self) {
        for path in self.watched_list.iter() {
            if let Err(error) = self.watcher.unwatch(path) {
                error!("Failed to unwatch shader path ({:?}): {error}", path);
            }
        }
    }

    fn watch_all(&mut self) {
        for path in self.watched_list.iter() {
            let path = static_path().join(path);
            self.watcher
                .watch(&path, notify::RecursiveMode::NonRecursive)
                .expect(&format!("Failed to start watching {:?}", &path));
        }
    }

    fn reload_list(&mut self) {
        self.watched_list.clear();
        self.watched_list
            .push(static_path().join(self.shader.path.clone()));
        self.watched_list.extend(self.assets.shader_library.clone());
    }

    fn reload_watch(&mut self) {
        // Stop watching old shader if the path has changed
        self.unwatch_all();
        self.reload_shader_library();
        futures::executor::block_on(self.shader.load(&self.geng));
        self.reload_list();
        self.watch_all();
    }

    fn reload_shader_library(&self) {
        futures::executor::block_on(load_shader_library(&self.geng, &static_path()))
            .expect("Failed to reload shader library");
    }
}

impl geng::State for ShaderEditState {
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
        ugli::clear(framebuffer, Some(Rgba::WHITE), None, None);

        self.view.draw_shader(framebuffer, &self.shader);
    }

    fn ui<'a>(&'a mut self, cx: &'a ui::Controller) -> Box<dyn ui::Widget + 'a> {
        (
            Text::new(
                "<Current shader>",
                cx.geng().default_font(),
                40.0,
                Rgba::WHITE,
            )
            .padding_horizontal(16.0)
            .center(),
            Text::new(
                self.shader.path.to_str().unwrap(),
                cx.geng().default_font(),
                40.0,
                Rgba::WHITE,
            )
            .padding_horizontal(16.0)
            .center(),
        )
            .column()
            .background_color(Rgba::try_from("#1491d477").unwrap())
            .uniform_padding(16.0)
            .align(vec2(0.0, 1.0))
            .boxed()
    }
}
