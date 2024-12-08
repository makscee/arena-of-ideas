use std::str::FromStr;

use bevy::{
    log::{debug, error, info},
    utils::hashbrown::HashMap,
};
use colored::{Colorize, CustomColor};
use ecolor::Hsva;
use egui::{text::LayoutJob, Label, Response, Style, TextFormat, Widget, WidgetText};
use itertools::Itertools;
use once_cell::sync::OnceCell;

use super::*;

pub type Cstr = String;

pub trait CstrTrait {
    fn widget(&self, a: f32, ui: &mut Ui) -> WidgetText;
    fn job(&self, a: f32, ui: &mut Ui) -> LayoutJob;
    fn label(&self, ui: &mut Ui) -> Response;
    fn label_alpha(&self, a: f32, ui: &mut Ui) -> Response;
    fn as_label(&self, ui: &mut Ui) -> Label;
    fn as_label_alpha(&self, a: f32, ui: &mut Ui) -> Label;
    fn button(self, ui: &mut Ui) -> Response;
    fn as_button(self) -> Button;
    fn get_text(&self) -> String;
    fn to_colored(&self) -> String;
    fn print(&self);
    fn info(&self);
    fn debug(&self);
    fn inject_vars(self, f: impl Fn(VarName) -> Option<VarValue>) -> Self;
}

impl CstrTrait for Cstr {
    fn widget(&self, a: f32, ui: &mut Ui) -> WidgetText {
        cstr_parse(&self.to_string(), a, ui.style())
    }
    fn job(&self, a: f32, ui: &mut Ui) -> LayoutJob {
        let mut job = LayoutJob::default();
        cstr_parse_into_job(&self, a, &mut job, ui.style());
        job
    }
    fn label(&self, ui: &mut Ui) -> Response {
        self.as_label(ui)
            .selectable(false)
            .wrap_mode(egui::TextWrapMode::Extend)
            .ui(ui)
    }
    fn label_alpha(&self, a: f32, ui: &mut Ui) -> Response {
        self.as_label_alpha(a, ui).ui(ui)
    }
    fn as_label(&self, ui: &mut Ui) -> Label {
        self.as_label_alpha(1.0, ui)
    }
    fn as_label_alpha(&self, a: f32, ui: &mut Ui) -> Label {
        Label::new(self.widget(a, ui))
    }
    fn button(self, ui: &mut Ui) -> Response {
        self.as_button().ui(ui)
    }
    fn as_button(self) -> Button {
        Button::new(self.clone())
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
                .map(|v| format!("[s [vd {var}:]][vb [b {v}]]"))
                .unwrap_or(format!("[vb {var}]"));
            self.replace_range(p..(p + var.len() + 1), &replace);
        }
        self
    }
}

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
#[derive(Default)]
struct StyleState {
    stack: Vec<CstrStyle>,
}

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum CstrStyle {
    Color(Color32),
    Bold,
    Small,
    Heading,
    Heading2,
}

impl CstrStyle {
    fn get_color(&self) -> Option<Color32> {
        match self {
            CstrStyle::Color(c) => Some(*c),
            _ => None,
        }
    }
    pub fn get_style(&self) -> Option<TextStyle> {
        match self {
            Self::Small => Some(TextStyle::Small),
            Self::Bold => Some(TextStyle::Name("Bold".into())),
            Self::Heading => Some(TextStyle::Heading),
            Self::Heading2 => Some(TextStyle::Name("Heading2".into())),
            _ => None,
        }
    }
    pub fn get_font(&self, style: &Style) -> Option<FontId> {
        self.get_style()
            .and_then(|s| style.text_styles.get(&s).cloned())
    }
}

static STRING_STYLE_MAP: OnceCell<HashMap<&'static str, CstrStyle>> = OnceCell::new();
static STYLE_STRING_MAP: OnceCell<HashMap<CstrStyle, &'static str>> = OnceCell::new();
pub fn init_style_map() {
    let pairs = [
        ("b", CstrStyle::Bold),
        ("s", CstrStyle::Small),
        ("h", CstrStyle::Heading),
        ("h2", CstrStyle::Heading2),
        ("red", CstrStyle::Color(RED)),
        ("green", CstrStyle::Color(GREEN)),
        ("yellow", CstrStyle::Color(YELLOW)),
        ("vd", CstrStyle::Color(VISIBLE_DARK)),
        ("vl", CstrStyle::Color(VISIBLE_LIGHT)),
        ("vb", CstrStyle::Color(VISIBLE_BRIGHT)),
    ];
    STRING_STYLE_MAP.set(HashMap::from_iter(pairs)).unwrap();
    STYLE_STRING_MAP
        .set(HashMap::from_iter(
            pairs.into_iter().map(|(str, style)| (style, str)),
        ))
        .unwrap();
}

impl CstrStyle {
    fn from_str(value: &str) -> Option<Self> {
        STRING_STYLE_MAP.get().unwrap().get(value).copied()
    }
    fn to_str(self) -> &'static str {
        STYLE_STRING_MAP.get().unwrap().get(&self).unwrap()
    }
}

impl StyleState {
    fn append(&self, text: &mut String, alpha: f32, job: &mut LayoutJob, style: &Style) {
        if text.is_empty() {
            return;
        }
        let color = self
            .stack
            .iter()
            .rev()
            .find_map(|s| s.get_color())
            .unwrap_or_else(|| style.visuals.widgets.inactive.fg_stroke.color)
            .gamma_multiply(alpha);
        let font_id = self
            .stack
            .iter()
            .rev()
            .find_map(|s| s.get_font(style))
            .unwrap_or_default();
        job.append(
            text,
            0.0,
            TextFormat {
                font_id,
                color,
                ..default()
            },
        );
        text.clear();
    }
    fn push(&mut self, style: CstrStyle) {
        self.stack.push(style);
    }
    fn push_token(&mut self, token: &str) {
        match CstrStyle::from_str(token) {
            Some(v) => self.stack.push(v),
            None => error!("Failed to parse token: {token}"),
        }
    }
    fn pop(&mut self) {
        if self.stack.is_empty() {
            error!("Tried to pop empty style stack");
        } else {
            self.stack.pop();
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
enum ParseState {
    Token,
    Text,
    HexColor,
}

pub fn cstr_parse(s: &str, alpha: f32, style: &Style) -> WidgetText {
    let mut job = LayoutJob::default();
    cstr_parse_into_job(s, alpha, &mut job, style);
    WidgetText::LayoutJob(job)
}
fn cstr_parse_into_job(s: &str, alpha: f32, job: &mut LayoutJob, style: &Style) {
    let mut cur = String::new();
    let mut style_state: StyleState = default();
    let mut parse_state = ParseState::Text;
    for c in s.chars() {
        match c {
            '[' => {
                style_state.append(&mut cur, alpha, job, style);
                parse_state = ParseState::Token;
            }
            ']' => {
                if parse_state == ParseState::Token {
                    let s = cur.cstr_s(CstrStyle::Bold);
                    cstr_parse_into_job(&s, alpha, job, style);
                    parse_state = ParseState::Text;
                    cur.clear();
                } else {
                    style_state.append(&mut cur, alpha, job, style);
                    style_state.pop();
                }
            }
            '#' => {
                if parse_state == ParseState::Token {
                    parse_state = ParseState::HexColor;
                }
                cur.push(c);
            }
            ' ' => {
                match parse_state {
                    ParseState::Token => {
                        style_state.push_token(&cur);
                        cur.clear();
                    }
                    ParseState::HexColor => {
                        match Color32::from_hex(&cur) {
                            Ok(c) => style_state.push(CstrStyle::Color(c)),
                            Err(e) => error!("Failed to parse hex color \"{cur}\": {e:?}"),
                        };
                        cur.clear();
                    }
                    ParseState::Text => cur.push(c),
                };
                parse_state = ParseState::Text;
            }
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        style_state.append(&mut cur, alpha, job, style);
    }
}

impl ToCstr for String {
    fn cstr(&self) -> Cstr {
        self.clone()
    }
}
impl ToCstr for str {
    fn cstr(&self) -> Cstr {
        self.to_owned()
    }
}
impl ToCstr for u32 {
    fn cstr(&self) -> Cstr {
        self.to_string().cstr_c(VISIBLE_LIGHT)
    }
}
impl ToCstr for i32 {
    fn cstr(&self) -> Cstr {
        self.to_string().cstr()
    }
    fn cstr_expanded(&self) -> Cstr {
        match self.signum() {
            1 => format!("+{self}").cstr_c(GREEN),
            -1 => format!("{self}").cstr_c(RED),
            _ => format!("{self}").cstr(),
        }
    }
}
impl ToCstr for VarName {
    fn cstr(&self) -> Cstr {
        self.as_ref().into()
    }
}
impl ToCstr for VarValue {
    fn cstr(&self) -> Cstr {
        match self {
            _ => self.to_string().cstr(),
        }
    }
}
