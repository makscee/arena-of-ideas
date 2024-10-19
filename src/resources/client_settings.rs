use bevy::{
    ecs::event::EventReader,
    window::{PresentMode, WindowResized},
};

use super::*;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct ClientSettings {
    pub servers: HashMap<&'static str, (&'static str, &'static str)>,
    pub active_server: &'static str,
    pub dev_mode: bool,

    pub window_mode: WindowMode,
    pub resolution: Vec2,
    pub vsync: bool,
    pub dark_theme: bool, //Plankton

    pub animation_time: f32,
    pub volume_music: f32,
    pub volume_fx: f32,
}

impl Default for ClientSettings {
    fn default() -> Self {
        Self {
            servers: HashMap::from([
                ("prod", ("http://161.35.88.206:3000", "aoi_prod")),
                ("dev", ("http://161.35.88.206:3000", "aoi_dev")),
            ]),
            active_server: "",
            dev_mode: false,
            window_mode: default(),
            vsync: false,
            dark_theme: true, //Plankton
            resolution: vec2(1280.0, 720.0),
            animation_time: 0.3,
            volume_music: 0.5,
            volume_fx: 1.0,
        }
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

static CLIENT_SETTINGS: OnceCell<RwLock<ClientSettings>> = OnceCell::new();
const CLIENT_SETTINGS_FILE: &str = "client_settings.ron";

fn path() -> PathBuf {
    let mut path = home_dir_path();
    path.push(CLIENT_SETTINGS_FILE);
    path
}
pub fn load_client_settings() {
    let mut cs = if let Some(cs) = std::fs::read_to_string(&path())
        .ok()
        .and_then(|d| ron::from_str::<ClientSettings>(d.leak()).ok())
    {
        cs
    } else {
        ClientSettings::default().save_to_file()
    };
    if cs.active_server.is_empty() {
        if cfg!(debug_assertions) {
            cs.active_server = "dev";
        } else {
            cs.active_server = "prod";
        }
    }
    cs.save_to_cache();
}
pub fn client_settings() -> std::sync::RwLockReadGuard<'static, ClientSettings> {
    CLIENT_SETTINGS.get_or_init(|| default()).read().unwrap()
}
pub fn current_server() -> (&'static str, &'static str) {
    let cs = client_settings();
    cs.servers[cs.active_server]
}
pub fn is_dev_mode() -> bool {
    client_settings().dev_mode
}

impl ClientSettings {
    pub fn save_to_cache(self) {
        *CLIENT_SETTINGS.get_or_init(|| default()).write().unwrap() = self;
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
            };
            AudioPlugin::set_music_volume(self.volume_music, world);
        }

        self.save_to_cache();
        let mut bg_res = world.resource_mut::<ClearColor>();
        bg_res.0 = emptiness().to_color();
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
