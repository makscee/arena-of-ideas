use crate::assets::*;

use super::*;

use geng::prelude::*;
use geng::ui::*;
use std::{path::PathBuf, sync::mpsc::Receiver};

use notify::{DebouncedEvent, RecommendedWatcher, Watcher};

type Time = R32;

pub struct ShaderEditState {
    geng: Geng,
    view: Rc<View>,
    time: Time,
    shaders_list: Vec<ShaderProgram>,
    shader_index: usize,
    receiver: Receiver<DebouncedEvent>,
    watcher: RecommendedWatcher,
    watched_list: Vec<PathBuf>,
    shader_library: Vec<PathBuf>,
    system_shaders: SystemShaders,
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
            watched_list: default(),
            watcher,
            shaders_list: default(),
            shader_index: 0,
            view,
            receiver: rx,
            system_shaders: assets.clone().system_shaders.clone(),
            shader_library: default(),
        };

        state.move_shaders_to_list();
        state.reload_watched_list();
        state.watch_all();
        state
    }

    fn get_current_shader(&self) -> &ShaderProgram {
        &self.shaders_list[self.shader_index]
    }

    fn switch_shader(&mut self, delta: i32) {
        self.shader_index = ((self.shader_index + self.shaders_list.len()) as i32 + delta) as usize
            % self.shaders_list.len();
        self.reload_watch();
    }

    fn handle_notify(&mut self, event: notify::DebouncedEvent) {
        debug!("Notify event: {event:?}");
        match event {
            DebouncedEvent::NoticeWrite(_path)
            // | DebouncedEvent::Write(_path)
            | DebouncedEvent::Create(_path) => self.reload_watch(),
            DebouncedEvent::NoticeRemove(_path) => {
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

    fn reload_watched_list(&mut self) {
        self.watched_list.clear();
        self.watched_list
            .push(static_path().join(self.get_current_shader().path.clone()));
        self.watched_list
            .push(static_path().join("shaders/system/config.json"));
        self.watched_list.extend(self.shader_library.clone());
    }

    fn reload_watch(&mut self) {
        self.unwatch_all();
        self.reload_shader_library();
        self.reload_system_shaders();
        self.move_shaders_to_list();
        futures::executor::block_on(self.shaders_list[self.shader_index].load(&self.geng));
        self.reload_watched_list();
        self.watch_all();
    }

    fn move_shaders_to_list(&mut self) {
        self.shaders_list = vec![
            self.system_shaders.unit.clone(),
            self.system_shaders.field.clone(),
        ];
    }

    fn reload_shader_library(&mut self) {
        self.shader_library =
            futures::executor::block_on(load_shader_library(&self.geng, &static_path()))
                .expect("Failed to reload shader library");
    }

    fn reload_system_shaders(&mut self) {
        match futures::executor::block_on(load_system_shaders(&self.geng, &static_path())) {
            Ok(list) => {
                self.system_shaders = list;
            }
            Err(error) => {
                error!("Failed to load system shaders {}", error);
            }
        };
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

        self.view
            .render
            .draw_shader(framebuffer, &self.get_current_shader());
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
                self.get_current_shader().path.to_str().unwrap(),
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

    fn handle_event(&mut self, event: Event) {
        match event {
            Event::KeyDown { key } => match key {
                Key::Left => self.switch_shader(-1),
                Key::Right => self.switch_shader(1),
                _ => {}
            },
            _ => {}
        }
    }
}
