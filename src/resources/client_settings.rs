use bevy::{
    ecs::event::EventReader,
    window::{PresentMode, VideoModeSelection, WindowResized},
};

use super::*;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ClientSettings {
    pub servers: HashMap<&'static str, (&'static str, &'static str)>,
    pub active_server: &'static str,
    pub dev_mode: bool,
    pub auto_login: bool,

    pub window_mode: WindowMode,
    pub resolution: Vec2,
    pub vsync: bool,

    pub animation_time: f32,
    pub volume_master: f32,
    pub volume_music: f32,
    pub volume_fx: f32,

    pub theme: Colorix,
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            servers: HashMap::from([
                ("prod", ("http://89.19.217.60:3000", "aoiprod")),
                ("dev", ("http://89.19.217.60:3000", "aoidev")),
            ]),
            active_server: "dev",
            dev_mode: false,
            auto_login: false,
            window_mode: default(),
            vsync: false,
            resolution: vec2(1280.0, 720.0),
            animation_time: 0.3,
            volume_master: 0.6,
            volume_music: 0.5,
            volume_fx: 1.0,
            theme: Colorix::new(GRAY, true),
        }
    }
}

impl PersistentData for ClientSettings {
    fn file_name() -> &'static str {
        "client_settings"
    }
}

#[derive(
    Default, Serialize, Deserialize, Debug, Copy, Clone, PartialEq, EnumString, EnumIter, Display,
)]
pub enum WindowMode {
    #[default]
    Windowed,
    FullScreen,
    BorderlessFullScreen,
}

pub fn current_server() -> (&'static str, &'static str) {
    let cs = &pd().client_settings;
    cs.servers[cs.active_server]
}
pub fn is_dev_mode() -> bool {
    pd().client_settings.dev_mode
}

impl ClientSettings {
    pub fn apply(self, world: &mut World) {
        if let Some(mut window) = world
            .query::<&mut bevy::window::Window>()
            .iter_mut(world)
            .next()
        {
            window.mode = match self.window_mode {
                WindowMode::Windowed => bevy::window::WindowMode::Windowed,
                WindowMode::FullScreen => bevy::window::WindowMode::Fullscreen(
                    bevy::window::MonitorSelection::Current,
                    VideoModeSelection::Current,
                ),
                WindowMode::BorderlessFullScreen => bevy::window::WindowMode::BorderlessFullscreen(
                    bevy::window::MonitorSelection::Current,
                ),
            };
            window
                .resolution
                .set(self.resolution.x.max(100.0), self.resolution.y.max(100.0));
            window.present_mode = match self.vsync {
                true => PresentMode::AutoVsync,
                false => PresentMode::AutoNoVsync,
            };
            AudioPlugin::set_music_volume(self.music_volume(), world);
        }
    }
    pub fn music_volume(&self) -> f32 {
        self.volume_music * self.volume_master
    }
    pub fn fx_volume(&self) -> f32 {
        self.volume_fx * self.volume_master
    }
}

pub struct ClientSettingsPlugin;

impl Plugin for ClientSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loaded), setup)
            .add_systems(Update, on_resize);
    }
}

fn setup(world: &mut World) {
    let cs = pd().client_settings.clone();
    cs.apply(world);
}
fn on_resize(mut resize_reader: EventReader<WindowResized>) {
    for e in resize_reader.read() {
        debug!("Resize {e:?}");
        pd_mut(|data| {
            data.client_settings.resolution = vec2(e.width, e.height);
        });
    }
}
