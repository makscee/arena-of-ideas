use egui::Sense;

use super::*;

pub struct Button {
    name: String,
    title: Option<String>,
    icon: Option<Icon>,
    min_width: f32,
    enabled: bool,
    active: bool,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            name: default(),
            title: default(),
            enabled: true,
            active: false,
            icon: None,
            min_width: 0.0,
        }
    }
}

impl Button {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..default()
        }
    }
    pub fn color(self, color: Color32, ui: &mut Ui) -> Self {
        let style = ui.style_mut();
        style.visuals.widgets.inactive.fg_stroke.color = color;
        self
    }
    pub fn gray(self, ui: &mut Ui) -> Self {
        self.color(VISIBLE_DARK, ui)
    }
    pub fn red(self, ui: &mut Ui) -> Self {
        let style = ui.style_mut();
        style.visuals.widgets.inactive.fg_stroke.color = DARK_RED;
        style.visuals.widgets.hovered.fg_stroke.color = RED;
        self
    }
    pub fn bg(self, ui: &mut Ui) -> Self {
        let style = ui.style_mut();
        style.visuals.widgets.inactive.weak_bg_fill = BG_LIGHT;
        style.visuals.widgets.hovered.weak_bg_fill = BG_LIGHT;
        self
    }
    pub fn set_bg(self, value: bool, ui: &mut Ui) -> Self {
        if value {
            self.bg(ui)
        } else {
            self
        }
    }
    pub fn title(mut self, text: String) -> Self {
        self.title = Some(text);
        self
    }
    pub fn credits_cost(mut self, cost: i64) -> Self {
        self.enabled = can_afford(cost);
        self.title = Some(self.name.clone());
        self.name = {
            let mut c = cost.to_string().cstr_c(VISIBLE_LIGHT)
                + &format!(" {CREDITS_SYM}").cstr_cs(YELLOW, CstrStyle::Bold);
            if !self.enabled {
                c = c.cstr_c(VISIBLE_DARK);
            }
            c
        };
        self
    }
    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }
    pub fn min_width(mut self, width: f32) -> Self {
        self.min_width = width;
        self
    }
    pub fn enabled(mut self, value: bool) -> Self {
        self.enabled = value;
        self
    }
    pub fn active(mut self, value: bool) -> Self {
        self.active = value;
        self
    }
    pub fn ui(self, ui: &mut Ui) -> Response {
        if let Some(title) = self.title {
            title.label(ui);
        }
        let mut job = self.name.job(1.0, ui);
        let mut replace_color = |c: Color32| {
            job.sections[0].format.color = c;
            job.sections[0].byte_range = 0..job.text.len();
            job.sections.truncate(1);
        };
        let style = ui.style_mut();
        if !self.enabled {
            style.visuals.widgets.noninteractive.bg_stroke.color = TRANSPARENT;
            style.visuals.widgets.noninteractive.fg_stroke.color = VISIBLE_DARK;
            replace_color(VISIBLE_DARK);
        } else if self.active {
            style.visuals.widgets.inactive.fg_stroke.color = YELLOW;
            style.visuals.widgets.hovered.fg_stroke.color = YELLOW;
            style.visuals.widgets.inactive.bg_stroke.color = YELLOW;
            style.visuals.widgets.hovered.bg_stroke.color = YELLOW;
            replace_color(YELLOW);
        }
        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::hover()
        };
        let r = if let Some(icon) = self.icon {
            egui::ImageButton::new(icon.image()).sense(sense).ui(ui)
        } else {
            egui::Button::new(WidgetText::LayoutJob(job))
                .wrap_mode(egui::TextWrapMode::Extend)
                .sense(sense)
                .min_size(egui::vec2(self.min_width, 0.0))
                .ui(ui)
        };
        if r.clicked() {
            AudioPlugin::queue_sound(SoundEffect::Click);
        }
        ui.reset_style();
        r
    }
}
