use std::{cell::LazyCell, ops::Deref, str::FromStr, sync::Arc};

use bevy::{
    color::Color,
    log::{debug, error, info},
    math::{vec2, Vec2},
    utils::hashbrown::HashMap,
};
use colored::{Colorize, CustomColor};
use ecolor::Hsva;
use egui::{text::LayoutJob, Galley, Label, Response, Style, TextFormat, Widget, WidgetText};
use itertools::Itertools;
use once_cell::sync::OnceCell;
use utils_client::ToC32;

use super::*;

pub type Cstr = String;

pub trait CstrTrait {
    fn widget(&self, a: f32, style: &Style) -> WidgetText;
    fn job(&self, a: f32, style: &Style) -> LayoutJob;
    fn label(&self, ui: &mut Ui) -> Response;
    fn label_w(&self, ui: &mut Ui) -> Response;
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
    fn galley(self, alpha: f32, ui: &mut Ui) -> Arc<Galley>;
}

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
    fn galley(self, alpha: f32, ui: &mut Ui) -> Arc<Galley> {
        let mut job = LayoutJob::default();
        cstr_parse_into_job(&self, alpha, &mut job, ui.style());
        ui.fonts(|r| r.layout_job(job))
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

static STRING_STYLE_MAP: OnceCell<Mutex<HashMap<&'static str, CstrStyle>>> = OnceCell::new();
static STYLE_STRING_MAP: OnceCell<Mutex<HashMap<CstrStyle, &'static str>>> = OnceCell::new();
pub fn init_style_map(colorix: &Colorix) {
    let pairs = [
        ("b", CstrStyle::Bold),
        ("s", CstrStyle::Small),
        ("h", CstrStyle::Heading),
        ("h2", CstrStyle::Heading2),
        ("red", CstrStyle::Color(RED)),
        ("green", CstrStyle::Color(GREEN)),
        ("yellow", CstrStyle::Color(YELLOW)),
        (
            "tl",
            CstrStyle::Color(colorix.tokens_global().low_contrast_text()),
        ),
        (
            "th",
            CstrStyle::Color(colorix.tokens_global().high_contrast_text()),
        ),
    ];
    *STRING_STYLE_MAP
        .get_or_init(|| Mutex::new(default()))
        .lock() = HashMap::from_iter(pairs);
    *STYLE_STRING_MAP
        .get_or_init(|| Mutex::new(default()))
        .lock() = HashMap::from_iter(pairs.into_iter().map(|(str, style)| (style, str)));
}

impl CstrStyle {
    fn from_str(value: &str) -> Option<Self> {
        STRING_STYLE_MAP.get().unwrap().lock().get(value).copied()
    }
    fn to_str(self) -> &'static str {
        STYLE_STRING_MAP.get().unwrap().lock().get(&self).unwrap()
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

impl<T: ToCstr> ToCstr for Vec<T> {
    fn cstr(&self) -> Cstr {
        self.into_iter().map(|i| i.cstr()).join(" ")
    }
}
impl<T: ToCstr> ToCstr for Box<T> {
    fn cstr(&self) -> Cstr {
        self.deref().cstr()
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
        self.to_string()
    }
}
impl ToCstr for u64 {
    fn cstr(&self) -> Cstr {
        self.to_string()
    }
}
impl ToCstr for f32 {
    fn cstr(&self) -> Cstr {
        format!("{self:.2}")
    }
}
impl ToCstr for f64 {
    fn cstr(&self) -> Cstr {
        format!("{self:.2}")
    }
}
impl ToCstr for i32 {
    fn cstr(&self) -> Cstr {
        self.to_string()
    }
    fn cstr_expanded(&self) -> Cstr {
        match self.signum() {
            1 => format!("+{self}").cstr_c(GREEN),
            -1 => format!("{self}").cstr_c(RED),
            _ => format!("{self}"),
        }
    }
}
impl ToCstr for bool {
    fn cstr(&self) -> Cstr {
        self.to_string()
    }
}
impl ToCstr for Vec2 {
    fn cstr(&self) -> Cstr {
        format!("({}, {})", self.x.cstr(), self.y.cstr())
    }
}
impl ToCstr for Color {
    fn cstr(&self) -> Cstr {
        self.c32().cstr()
    }
}
impl ToCstr for Color32 {
    fn cstr(&self) -> Cstr {
        self.to_hex().cstr_c(*self)
    }
}
impl ToCstr for HexColor {
    fn cstr(&self) -> Cstr {
        let s = &self.0;
        format!("[{s} {s}]")
    }
}
impl ToCstr for VarName {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr_cs(self.color(), CstrStyle::Small)
    }
}
impl ToCstr for VarValue {
    fn cstr(&self) -> Cstr {
        match self {
            _ => self.to_string().cstr(),
        }
    }
}
impl ToCstr for Expression {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr_c(self.color())
    }
    fn cstr_expanded(&self) -> Cstr {
        if let Some(description) = Descriptions::get(self) {
            return description.clone();
        }
        let inner = match self {
            Expression::One
            | Expression::Zero
            | Expression::PI
            | Expression::PI2
            | Expression::GT
            | Expression::UnitSize
            | Expression::AllUnits
            | Expression::AllAllyUnits
            | Expression::AllOtherAllyUnits
            | Expression::AdjacentAllyUnits
            | Expression::AdjacentBack
            | Expression::AdjacentFront
            | Expression::AllEnemyUnits
            | Expression::Owner
            | Expression::Target => String::default(),
            Expression::Var(v) => v.cstr(),
            Expression::V(v) => v.cstr(),
            Expression::S(v) => v.to_owned(),
            Expression::F(v) | Expression::FSlider(v) => v.cstr(),
            Expression::I(v) => v.cstr(),
            Expression::B(v) => v.cstr(),
            Expression::V2(x, y) => vec2(*x, *y).cstr(),
            Expression::C(c) => match Color32::from_hex(c) {
                Ok(color) => c.cstr_c(color),
                Err(e) => format!("{c} [s {e:?}]",).cstr_c(RED),
            },
            Expression::Sin(x)
            | Expression::Cos(x)
            | Expression::Even(x)
            | Expression::Abs(x)
            | Expression::Floor(x)
            | Expression::Ceil(x)
            | Expression::Fract(x)
            | Expression::UnitVec(x)
            | Expression::Rand(x)
            | Expression::RandomUnit(x)
            | Expression::ToF(x)
            | Expression::Sqr(x) => x.cstr_expanded(),
            Expression::Macro(a, b)
            | Expression::V2EE(a, b)
            | Expression::Sum(a, b)
            | Expression::Sub(a, b)
            | Expression::Mul(a, b)
            | Expression::Div(a, b)
            | Expression::Max(a, b)
            | Expression::Min(a, b)
            | Expression::Mod(a, b)
            | Expression::And(a, b)
            | Expression::Or(a, b)
            | Expression::Equals(a, b)
            | Expression::GreaterThen(a, b)
            | Expression::LessThen(a, b)
            | Expression::Fallback(a, b) => format!("{}, {}", a.cstr_expanded(), b.cstr_expanded()),
            Expression::Oklch(a, b, c) | Expression::If(a, b, c) => format!(
                "{}, {}, {}",
                a.cstr_expanded(),
                b.cstr_expanded(),
                c.cstr_expanded()
            ),
            Expression::StateVar(x, v) => format!("{}({})", x.cstr_expanded(), v.cstr_expanded()),
        };
        if inner.is_empty() {
            self.cstr()
        } else {
            format!("{}[tl (]{inner}[tl )]", self.cstr())
        }
    }
}

impl ToCstr for PainterAction {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr_c(CYAN)
    }
    fn cstr_expanded(&self) -> Cstr {
        let inner = match self {
            PainterAction::Circle(x)
            | PainterAction::Rectangle(x)
            | PainterAction::Text(x)
            | PainterAction::Hollow(x)
            | PainterAction::Translate(x)
            | PainterAction::Rotate(x)
            | PainterAction::ScaleMesh(x)
            | PainterAction::ScaleRect(x)
            | PainterAction::Alpha(x)
            | PainterAction::Feathering(x)
            | PainterAction::Color(x) => x.cstr_expanded(),
            PainterAction::Curve {
                thickness,
                curvature,
            } => format!(
                "{}, {}",
                thickness.cstr_expanded(),
                curvature.cstr_expanded()
            ),
            PainterAction::Repeat(x, a) => format!("{}, {}", x.cstr_expanded(), a.cstr_expanded()),
            PainterAction::List(vec) => vec.into_iter().map(|a| a.cstr_expanded()).join(", "),
            PainterAction::Paint => default(),
        };
        format!("{}({inner})", self.cstr())
    }
}
impl ToCstr for Material {
    fn cstr(&self) -> Cstr {
        format!("({})", self.0.iter().map(|a| a.cstr()).join(", "))
    }
    fn cstr_expanded(&self) -> Cstr {
        format!("({})", self.0.iter().map(|a| a.cstr_expanded()).join(", "))
    }
}
impl ToCstr for Trigger {
    fn cstr(&self) -> Cstr {
        self.as_ref().to_owned().cstr_c(self.color())
    }
}
impl ToCstr for Action {
    fn cstr(&self) -> Cstr {
        let inner_x = <Self as Injector<Expression>>::get_inner(self);
        let inner_a = <Self as Injector<Action>>::get_inner(self);
        let s = self.as_ref().to_owned().cstr_c(self.color());
        if !inner_x.is_empty() || !inner_a.is_empty() {
            let inner = inner_x
                .into_iter()
                .map(|x| x.cstr_expanded())
                .chain(inner_a.into_iter().map(|a| a.cstr_expanded()))
                .join(", ");
            format!("{s}[tl (]{inner}[tl )]")
        } else {
            s
        }
    }
}
impl ToCstr for Event {
    fn cstr(&self) -> Cstr {
        self.as_ref().to_owned()
    }
}
impl ToCstr for ExpressionError {
    fn cstr(&self) -> Cstr {
        format!("{self}").cstr_cs(RED, CstrStyle::Small)
    }
}
impl ToCstr for Reaction {
    fn cstr(&self) -> Cstr {
        format!("{}", self.trigger.cstr())
    }
}
