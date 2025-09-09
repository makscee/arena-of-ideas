pub mod parser;
pub mod style;
pub mod traits;

use std::sync::Arc;

use ecolor::Hsva;
pub use parser::*;
pub use style::*;

use super::*;
use crate::ui::widgets::Button;

pub type Cstr = String;

pub trait ToCstr {
    fn cstr(&self) -> Cstr;

    fn cstr_c(&self, color: Color32) -> Cstr {
        format!("[{} {}]", color.to_hex(), self.cstr())
    }

    fn cstr_s(&self, style: CstrStyle) -> Cstr {
        format!("[{} {}]", style.to_str(), self.cstr())
    }

    fn cstr_cs(&self, color: Color32, style: CstrStyle) -> Cstr {
        self.cstr().cstr_c(color).cstr_s(style)
    }

    fn cstr_expanded(&self) -> Cstr {
        self.cstr()
    }

    fn cstr_rainbow(&self) -> Cstr {
        let mut c: String = default();
        let chars = self.cstr().get_text().chars().collect_vec();
        let len = chars.len();
        for (i, char) in chars.into_iter().enumerate() {
            let h = i as f32 / len as f32 + gt().play_head() * 0.1;
            let color = Hsva::new(h.fract(), 1.0, 1.0, 1.0);
            c = c + &String::from(char).cstr_c(color.into());
        }
        c
    }
}

/// SFnCstr is an alias for ToCstr to follow the see module pattern
pub trait SFnCstr: ToCstr {}

/// Blanket implementation for all types that implement ToCstr
impl<T: ToCstr> SFnCstr for T {}

/// Original CstrTrait for widget functionality
pub trait CstrTrait {
    fn widget(&self, a: f32, style: &Style) -> WidgetText;
    fn job(&self, a: f32, style: &Style) -> LayoutJob;
    fn label(&self, ui: &mut Ui) -> Response;
    fn label_w(&self, ui: &mut Ui) -> Response;
    fn label_t(&self, ui: &mut Ui) -> Response;
    fn label_alpha(&self, a: f32, ui: &mut Ui) -> Response;
    fn as_label(&self, style: &Style) -> Label;
    fn as_label_alpha(&self, a: f32, style: &Style) -> Label;
    fn button(self, ui: &mut Ui) -> Response;
    fn as_button(self) -> Button;
    fn get_text(&self) -> String;
    fn to_colored(&self) -> String;
    fn print(&self);
    fn info(&self);
    fn debug(&self);
    fn inject_vars(self, f: impl Fn(VarName) -> Option<VarValue>) -> Self;
    fn galley(self, alpha: f32, ui: &mut Ui) -> Arc<egui::Galley>;
}

/// SFnCstrWidget is an alias for CstrTrait to follow the see module pattern
pub trait SFnCstrWidget: CstrTrait {}

/// Blanket implementation for all types that implement CstrTrait
impl<T: CstrTrait> SFnCstrWidget for T {}

impl CstrTrait for Cstr {
    fn widget(&self, a: f32, style: &Style) -> WidgetText {
        cstr_parse(&self.to_string(), a, style)
    }

    fn job(&self, a: f32, style: &Style) -> LayoutJob {
        let mut job = LayoutJob::default();
        cstr_parse_into_job(&self, a, &mut job, style);
        job
    }

    fn label(&self, ui: &mut Ui) -> Response {
        self.as_label(ui.style()).ui(ui)
    }

    fn label_w(&self, ui: &mut Ui) -> Response {
        self.as_label(ui.style()).wrap().ui(ui)
    }

    fn label_t(&self, ui: &mut Ui) -> Response {
        self.as_label(ui.style()).truncate().ui(ui)
    }

    fn label_alpha(&self, a: f32, ui: &mut Ui) -> Response {
        self.as_label_alpha(a, ui.style()).ui(ui)
    }

    fn as_label(&self, style: &Style) -> Label {
        self.as_label_alpha(1.0, style)
    }

    fn as_label_alpha(&self, a: f32, style: &Style) -> Label {
        Label::new(self.widget(a, style)).selectable(false)
    }

    fn button(self, ui: &mut Ui) -> Response {
        self.as_button().ui(ui)
    }

    fn as_button(self) -> Button {
        Button::new(self)
    }

    fn get_text(&self) -> String {
        let mut job: LayoutJob = default();
        cstr_parse_into_job(self, 1.0, &mut job, &default());
        job.text
    }

    fn to_colored(&self) -> String {
        let mut job: LayoutJob = default();
        cstr_parse_into_job(self, 1.0, &mut job, &default());
        let mut s = String::new();
        for egui::text::LayoutSection {
            leading_space: _,
            byte_range,
            format,
        } in job.sections
        {
            let text = &job.text[byte_range];
            let color = format.color;
            s += &text
                .custom_color(CustomColor {
                    r: color.r(),
                    g: color.g(),
                    b: color.b(),
                })
                .to_string();
        }
        s
    }

    fn print(&self) {
        println!("{}", self.to_colored())
    }

    fn info(&self) {
        info!("{}", self.to_colored())
    }

    fn debug(&self) {
        debug!("{}", self.to_colored())
    }

    fn inject_vars(mut self, f: impl Fn(VarName) -> Option<VarValue>) -> Self {
        while let Some(p) = self.find('$') {
            let mut var = String::new();
            for c in self[p + 1..].chars() {
                if c.is_alphabetic() {
                    var.push(c);
                } else {
                    break;
                }
            }
            let replace = VarName::from_str(&var)
                .ok()
                .and_then(|v| f(v))
                .and_then(|v| v.get_string().ok())
                .map(|v| format!("[s [tl {var}:]][th [b {v}]]"))
                .unwrap_or(format!("[th {var}]"));
            self.replace_range(p..(p + var.len() + 1), &replace);
        }
        self
    }

    fn galley(self, alpha: f32, ui: &mut Ui) -> Arc<egui::Galley> {
        let mut job = LayoutJob::default();
        cstr_parse_into_job(&self, alpha, &mut job, ui.style());
        ui.fonts(|r| r.layout_job(job))
    }
}
