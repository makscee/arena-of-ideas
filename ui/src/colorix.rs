use egui_colors::{
    tokens::{ColorTokens, ThemeColor},
    Theme,
};

use super::*;

#[derive(Resource, Default)]
pub struct Colorix {
    pub semantics: Vec<egui_colors::Colorix>,
}

#[repr(usize)]
enum Semantics {
    Global,
    Error,
    Success,
    Warning,
    Info,
}

impl Colorix {
    pub fn global(&mut self) -> &mut egui_colors::Colorix {
        &mut self.semantics[Semantics::Global as usize]
    }
    pub fn style_error(&mut self, ui: &mut Ui) {
        apply_style(&mut self.semantics[Semantics::Error as usize], ui);
    }
    pub fn style_success(&mut self, ui: &mut Ui) {
        apply_style(&mut self.semantics[Semantics::Success as usize], ui);
    }
    pub fn style_warning(&mut self, ui: &mut Ui) {
        apply_style(&mut self.semantics[Semantics::Warning as usize], ui);
    }
    pub fn style_info(&mut self, ui: &mut Ui) {
        apply_style(&mut self.semantics[Semantics::Info as usize], ui);
    }
}

static TOKENS: Mutex<Vec<ColorTokens>> = Mutex::new(Vec::new());
pub fn tokens_global() -> ColorTokens {
    TOKENS.lock().unwrap()[Semantics::Global as usize]
}
pub fn tokens_error() -> ColorTokens {
    TOKENS.lock().unwrap()[Semantics::Error as usize]
}
pub fn tokens_success() -> ColorTokens {
    TOKENS.lock().unwrap()[Semantics::Success as usize]
}
pub fn tokens_warning() -> ColorTokens {
    TOKENS.lock().unwrap()[Semantics::Warning as usize]
}
pub fn tokens_info() -> ColorTokens {
    TOKENS.lock().unwrap()[Semantics::Info as usize]
}

fn apply_style(colorix: &mut egui_colors::Colorix, ui: &mut Ui) {
    colorix.update_locally(ui);
    override_style(ui.style_mut());
}

fn override_style(style: &mut egui::Style) {
    style.visuals.widgets.active.corner_radius = CornerRadius::same(13);
    style.visuals.widgets.inactive.corner_radius = CornerRadius::same(13);
    style.visuals.widgets.hovered.corner_radius = CornerRadius::same(13);
    style.visuals.widgets.noninteractive.corner_radius = CornerRadius::same(13);
    style.visuals.widgets.open.corner_radius = CornerRadius::same(13);
}

pub fn setup_colorix(world: &mut World) {
    let ctx = &egui_context(world).unwrap();
    let theme_main: Theme = [ThemeColor::Custom([0; 3]); 12];
    let theme_error: Theme = [ThemeColor::Ruby; 12];
    let theme_success: Theme = [ThemeColor::Green; 12];
    let theme_warning: Theme = [ThemeColor::Orange; 12];
    let theme_info: Theme = [ThemeColor::Cyan; 12];
    let global = egui_colors::Colorix::global(ctx, theme_main);
    ctx.style_mut(|style| override_style(style));
    let semantics = [
        global,
        egui_colors::Colorix::local_from_style(theme_error, true),
        egui_colors::Colorix::local_from_style(theme_success, true),
        egui_colors::Colorix::local_from_style(theme_warning, true),
        egui_colors::Colorix::local_from_style(theme_info, true),
    ]
    .to_vec();
    *TOKENS.lock().unwrap() = semantics.iter().map(|s| s.tokens).collect();
    world.insert_resource(Colorix { semantics });
    world.insert_resource(bevy::render::camera::ClearColor(
        tokens_global().app_background().to_color(),
    ));
    init_style_map();
}

pub trait ColorixExt {
    fn colorix(&self) -> &Colorix;
    fn colorix_mut(&mut self) -> Mut<Colorix>;
}

impl ColorixExt for World {
    fn colorix(&self) -> &Colorix {
        &self.resource::<Colorix>()
    }
    fn colorix_mut(&mut self) -> Mut<Colorix> {
        self.resource_mut::<Colorix>()
    }
}
