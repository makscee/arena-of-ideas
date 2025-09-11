use super::*;
use crate::ui::core::Colorix;
use crate::ui::core::{GREEN, RED, YELLOW};
use egui::{Color32, FontId, Style, TextFormat, TextStyle, text::LayoutJob};
use log::error;
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use std::collections::HashMap;

#[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub enum CstrStyle {
    Color(Color32),
    Normal,
    Bold,
    Small,
    Heading,
    Heading2,
}

impl CstrStyle {
    pub fn get_color(&self) -> Option<Color32> {
        match self {
            CstrStyle::Color(c) => Some(*c),
            _ => None,
        }
    }

    pub fn get_style(&self) -> Option<TextStyle> {
        match self {
            Self::Normal => Some(TextStyle::Body),
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

    pub fn from_str(value: &str) -> Option<Self> {
        STRING_STYLE_MAP.get().unwrap().lock().get(value).copied()
    }

    pub fn to_str(self) -> &'static str {
        STYLE_STRING_MAP.get().unwrap().lock().get(&self).unwrap()
    }
}

#[derive(Default)]
pub struct StyleState {
    pub stack: Vec<CstrStyle>,
}

impl StyleState {
    pub fn append(&self, text: &mut String, alpha: f32, job: &mut LayoutJob, style: &Style) {
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

    pub fn push(&mut self, style: CstrStyle) {
        self.stack.push(style);
    }

    pub fn push_token(&mut self, token: &str) {
        match CstrStyle::from_str(token) {
            Some(v) => self.stack.push(v),
            None => error!("Failed to parse token: {token}"),
        }
    }

    pub fn pop(&mut self) {
        if self.stack.is_empty() {
            error!("Tried to pop empty style stack");
        } else {
            self.stack.pop();
        }
    }
}

static STRING_STYLE_MAP: OnceCell<Mutex<HashMap<&'static str, CstrStyle>>> = OnceCell::new();
static STYLE_STRING_MAP: OnceCell<Mutex<HashMap<CstrStyle, &'static str>>> = OnceCell::new();

pub fn init_style_map(colorix: &Colorix, style: &Style) {
    let pairs = [
        ("b", CstrStyle::Bold),
        ("s", CstrStyle::Small),
        ("n", CstrStyle::Normal),
        ("h", CstrStyle::Heading),
        ("h2", CstrStyle::Heading2),
        ("red", CstrStyle::Color(RED)),
        ("green", CstrStyle::Color(GREEN)),
        ("yellow", CstrStyle::Color(YELLOW)),
        ("tw", CstrStyle::Color(style.visuals.weak_text_color())),
        ("tl", CstrStyle::Color(colorix.low_contrast_text())),
        ("th", CstrStyle::Color(colorix.high_contrast_text())),
    ];
    *STRING_STYLE_MAP
        .get_or_init(|| Mutex::new(default()))
        .lock() = HashMap::from_iter(pairs);
    *STYLE_STRING_MAP
        .get_or_init(|| Mutex::new(default()))
        .lock() = HashMap::from_iter(pairs.into_iter().map(|(str, style)| (style, str)));
}

#[derive(PartialEq, Clone, Copy)]
pub enum ParseState {
    Token,
    Text,
    HexColor,
}
