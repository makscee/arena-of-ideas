use std::sync::RwLock;

use bevy::{
    ecs::event::EventReader,
    window::{PresentMode, WindowResized},
};

use super::*;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ClientSettings {
    pub dev_server: (String, String),
    pub prod_server: (String, String),
    pub dev_mode: bool,

    pub window_mode: WindowMode,
    pub resolution: Vec2,
    pub vsync: bool,
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

static CLIENT_SETTINGS: RwLock<ClientSettings> = RwLock::new(ClientSettings {
    dev_server: (String::new(), String::new()),
    prod_server: (String::new(), String::new()),
    dev_mode: false,
    window_mode: WindowMode::Windowed,
    resolution: vec2(100.0, 100.0),
    vsync: false,
});
const CLIENT_SETTINGS_FILE: &str = "client_settings.ron";

fn path() -> PathBuf {
    let mut path = home_dir_path();
    path.push(CLIENT_SETTINGS_FILE);
    path
}
pub fn load_client_settings() {
    let cs = if let Some(cs) = std::fs::read_to_string(&path())
        .ok()
        .and_then(|d| ron::from_str::<ClientSettings>(&d).ok())
    {
        cs
    } else {
        ClientSettings::default().save_to_file()
    };
    cs.save_to_cache();
}
pub fn client_settings() -> std::sync::RwLockReadGuard<'static, ClientSettings> {
    CLIENT_SETTINGS.read().unwrap()
}
pub fn is_dev_mode() -> bool {
    client_settings().dev_mode
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            dev_server: ("http://161.35.88.206:3000".into(), "aoi_dev".into()),
            prod_server: ("http://161.35.88.206:3000".into(), "aoi_prod".into()),
            dev_mode: false,
            window_mode: default(),
            vsync: false,
            resolution: vec2(1280.0, 720.0),
        }
    }
}

impl ClientSettings {
    pub fn save_to_cache(self) {
        *CLIENT_SETTINGS.write().unwrap() = self;
    }
    pub fn save_to_file(self) -> Self {
        match std::fs::write(
            path(),
            to_string_pretty(&self, PrettyConfig::new())
                .expect("Failed to serialize default client settings"),
        ) {
            Ok(_) => {
                info!("Store successful {self:?}")
            }
            Err(e) => {
                error!("Store error: {e}")
            }
        }
        self
    }
    pub fn apply(self, world: &mut World) {
        if let Some(mut window) = world
            .query::<&mut bevy::window::Window>()
            .iter_mut(world)
            .next()
        {
            window.mode = match self.window_mode {
                WindowMode::Windowed => bevy::window::WindowMode::Windowed,
                WindowMode::FullScreen => bevy::window::WindowMode::Fullscreen,
                WindowMode::BorderlessFullScreen => bevy::window::WindowMode::BorderlessFullscreen,
            };
            window
                .resolution
                .set(self.resolution.x.max(100.0), self.resolution.y.max(100.0));
            window.present_mode = match self.vsync {
                true => PresentMode::AutoVsync,
                false => PresentMode::AutoNoVsync,
            }
        }
        self.save_to_cache();
    }
}

pub struct ClientSettingsPlugin;

impl Plugin for ClientSettingsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, on_resize);
    }
}

fn setup(world: &mut World) {
    let cs = client_settings().clone();
    cs.apply(world);
}
fn on_resize(mut resize_reader: EventReader<WindowResized>) {
    for e in resize_reader.read() {
        debug!("Resize {e:?}");
        let mut cs = client_settings().clone();
        cs.resolution = vec2(e.width, e.height);
        cs.save_to_file().save_to_cache();
    }
}
