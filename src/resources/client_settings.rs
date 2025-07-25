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

#[macro_export]
macro_rules! settings_editor {
    ($settings:expr, $ui:expr) => {
        egui::Grid::new("settings_grid")
            .num_columns(2)
            .spacing([40.0, 4.0])
            .show($ui, |ui| {
                settings_field!(server_select, $settings, ui);
                settings_field!($settings, dev_mode, "Dev Mode", ui);
                settings_field!($settings, auto_login, "Auto Login", ui);
                ui.end_row();

                settings_field!($settings, window_mode, "Window Mode", ui);
                settings_field!($settings, resolution, "Resolution", ui);
                settings_field!($settings, vsync, "VSync", ui);
                ui.end_row();

                settings_field!($settings, animation_time, "Animation Time", ui);
                settings_field!($settings, volume_master, "Master Volume", ui);
                settings_field!($settings, volume_music, "Music Volume", ui);
                ui.end_row();

                settings_field!($settings, volume_fx, "FX Volume", ui);
                settings_field!($settings, theme, "Theme", ui);
                ui.end_row();
            });
    };
}

#[macro_export]
macro_rules! settings_field {
    ($settings:expr, $field:ident, $label:expr, $ui:expr) => {
        $ui.label($label);
        if $settings.$field.show_mut(&Context::default(), $ui) {
            pd_mut(|d| d.client_settings.$field = $settings.$field.clone());
        }
        $ui.end_row();
    };
    (server_select, $settings:expr, $ui:expr) => {
        $ui.label("Active Server");
        let mut current_server = $settings.active_server;
        let server_names: Vec<&'static str> = $settings.servers.keys().copied().collect();
        let mut changed = false;
        egui::ComboBox::from_id_salt("server_select")
            .selected_text(current_server)
            .show_ui($ui, |ui| {
                for &server_name in &server_names {
                    if ui
                        .selectable_value(&mut current_server, server_name, server_name)
                        .changed()
                    {
                        changed = true;
                    }
                }
            });
        if changed {
            $settings.active_server = current_server;
            pd_mut(|d| d.client_settings.active_server = current_server);
        }
        $ui.end_row();
    };
}

impl Show for &'static str {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.cstr().label(ui);
    }
    fn show_mut(&mut self, _: &Context, _ui: &mut Ui) -> bool {
        false
    }
}

impl ToCstr for WindowMode {
    fn cstr(&self) -> Cstr {
        self.to_string()
    }
}

impl Show for WindowMode {
    fn show(&self, _: &Context, ui: &mut Ui) {
        self.to_string().cstr().label(ui);
    }
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        Selector::new("").ui_enum(self, ui)
    }
}

impl Show for Colorix {
    fn show(&self, _: &Context, ui: &mut Ui) {
        "Theme".cstr_c(self.color(0)).label(ui);
    }
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        let mut color = self.raw_colors[0];
        if ui.color_edit_button_srgba(&mut color).changed() {
            for c in &mut self.raw_colors {
                *c = color;
            }
            self.generate_scale();
            self.apply(ui.ctx());
            true
        } else {
            false
        }
    }
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
    Default,
    Serialize,
    Deserialize,
    Debug,
    Copy,
    Clone,
    PartialEq,
    EnumString,
    EnumIter,
    Display,
    AsRefStr,
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
