use bevy::{
    ecs::event::EventReader,
    window::{PresentMode, VideoModeSelection, WindowResized},
};
use bevy_egui::egui::color_picker::color_picker_hsva_2d;
use ecolor::Hsva;
use settings_derive::Settings;

use super::*;

/// Settings configuration using attribute-based UI generation.
///
/// Supported attributes:
/// - `#[setting(slider(default, min, max), "Label")]` - Creates a slider widget
/// - `#[setting(checkbox(default), "Label")]` - Creates a checkbox widget
/// - `#[setting(selector(fn_name), "Label")]` - Creates a dropdown with options from fn_name()
/// - `#[setting(enum, "Label")]` - Creates an enum selector widget
/// - `#[setting(show, "Label")]` - Uses the Show trait's show_mut method
///
/// Example usage:
/// ```rust
/// #[setting(slider(0.5, 0.0, 1.0), "Volume")]
/// pub volume: f32,
///
/// #[setting(checkbox(true), "Enable Feature")]
/// pub enabled: bool,
///
/// #[setting(selector(get_options), "Choice")]
/// pub selection: String,
///
/// #[setting(enum, "Mode")]
/// pub mode: GameMode,
///
/// #[setting(show, "Custom Widget")]
/// pub custom: CustomType,
/// ```
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Settings)]
pub struct ClientSettings {
    pub servers: HashMap<&'static str, (&'static str, &'static str)>,
    #[setting(selector(get_server_options), "Active Server")]
    pub active_server: &'static str,
    #[setting(checkbox(false), "Dev Mode")]
    pub dev_mode: bool,
    #[setting(checkbox(false), "Auto Login")]
    pub auto_login: bool,

    #[setting(enum, "Window Mode")]
    pub window_mode: WindowMode,
    #[setting(selector(get_resolution_options), "Resolution")]
    pub resolution: Vec2,
    #[setting(checkbox(false), "VSync")]
    pub vsync: bool,

    #[setting(slider(0.3, 0.0, 2.0), "Animation Time")]
    pub animation_time: f32,
    #[setting(slider(0.6, 0.0, 1.0), "Master Volume")]
    pub volume_master: f32,
    #[setting(slider(0.5, 0.0, 1.0), "Music Volume")]
    pub volume_music: f32,
    #[setting(slider(1.0, 0.0, 1.0), "FX Volume")]
    pub volume_fx: f32,

    #[setting(checkbox(true), "Show Debug Info")]
    pub show_debug_info: bool,

    #[setting(show, "Theme")]
    pub theme: Colorix,
}

#[macro_export]
macro_rules! settings_editor {
    ($settings:expr, $ui:expr) => {
        $settings.generate_settings_ui($ui);

        $ui.separator();
        $ui.columns(2, |ui| {
            ui[0].vertical_centered_justified(|ui| {
                if ui.button("Save").clicked() {
                    pd_save_settings();
                    ui.close_menu();
                }
            });
            ui[1].vertical_centered_justified(|ui| {
                if ui.button("Discard").clicked() {
                    pd_discard_settings();
                    ui.close_menu();
                }
            });
        });
    };
}

fn get_server_options() -> Vec<&'static str> {
    let cs = &pd().client_settings;
    cs.servers.keys().copied().collect()
}

fn get_resolution_options() -> Vec<Vec2> {
    vec![
        vec2(1280.0, 720.0),
        vec2(1920.0, 1080.0),
        vec2(2560.0, 1440.0),
        vec2(3840.0, 2160.0),
    ]
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
        ui.menu_button("Theme".cstr_c(self.color(0)), |ui| {
            "Theme".cstr_c(self.color(0)).label(ui);
        });
    }
    fn show_mut(&mut self, _: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        let mut hsva = Hsva::from_srgba_unmultiplied(self.raw_colors[0].to_array());
        ui.horizontal(|ui| {
            for i in 0_usize..12 {
                let rect = ui.allocate_space(LINE_HEIGHT.v2()).1;
                ui.painter().rect_filled(rect, 0, colorix().color(i));
            }
        });
        if color_picker_hsva_2d(ui, &mut hsva, egui::color_picker::Alpha::Opaque) {
            let c = hsva.to_srgba_unmultiplied();
            let color = Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]);
            for c in &mut self.raw_colors {
                *c = color;
            }
            self.generate_scale();
            self.apply(ui.ctx());
            self.clone().save();
            changed = true;
        }
        changed
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
            show_debug_info: true,
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
            .add_systems(Update, (on_resize, apply_settings_changes));
    }
}

fn setup(world: &mut World) {
    let cs = pd().client_settings.clone();
    cs.apply(world);
}

fn apply_settings_changes(world: &mut World) {
    let cs = pd().client_settings.clone();
    let saved_cs = pd().saved_client_settings.clone();
    if cs != saved_cs {
        cs.apply(world);
    }
}

fn on_resize(mut resize_reader: EventReader<WindowResized>) {
    for e in resize_reader.read() {
        debug!("Resize {e:?}");
        pd_mut(|data| {
            data.client_settings.resolution = vec2(e.width, e.height);
        });
    }
}
