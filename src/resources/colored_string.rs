use bevy_egui::egui::Style;

use super::*;

#[derive(Clone, Debug, Default)]
pub struct ColoredString {
    pub lines: Vec<(String, Option<Color32>, ColoredStringStyle)>,
    extra_size: f32,
    extra_spacing: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub enum ColoredStringStyle {
    #[default]
    Normal,
    Small,
    Bold,
    Heading,
    Heading2,
}

impl ColoredStringStyle {
    fn get_font(&self, style: &Style) -> FontId {
        match self {
            ColoredStringStyle::Normal => TextStyle::Body,
            ColoredStringStyle::Small => TextStyle::Small,
            ColoredStringStyle::Bold => TextStyle::Name("Bold".into()),
            ColoredStringStyle::Heading => TextStyle::Heading,
            ColoredStringStyle::Heading2 => TextStyle::Name("Heading2".into()),
        }
        .resolve(style)
    }
}

impl ColoredString {
    pub fn simple(text: String) -> Self {
        Self {
            lines: [(text, None, default())].into(),
            extra_size: 0.0,
            extra_spacing: 0.0,
        }
    }

    pub fn new(text: String, color: Color32) -> Self {
        Self {
            lines: vec![(text, Some(color), default())],
            extra_size: 0.0,
            extra_spacing: 0.0,
        }
    }

    pub fn push_colored(&mut self, cstring: ColoredString) -> &mut Self {
        self.lines.extend(cstring.lines);
        self
    }
    pub fn push_colored_front(&mut self, mut cstring: ColoredString) -> &mut Self {
        mem::swap(self, &mut cstring);
        self.lines.extend(cstring.lines);
        self
    }

    pub fn push(&mut self, text: String, color: Color32) -> &mut Self {
        self.lines.push((text, Some(color), default()));
        self
    }

    pub fn push_styled(
        &mut self,
        text: String,
        color: Color32,
        style: ColoredStringStyle,
    ) -> &mut Self {
        self.lines.push((text, Some(color), style));
        self
    }

    pub fn set_style_ref(&mut self, style: ColoredStringStyle) -> &mut Self {
        self.lines.iter_mut().for_each(|(_, _, s)| *s = style);
        self
    }
    pub fn set_style(mut self, style: ColoredStringStyle) -> Self {
        self.set_style_ref(style);
        self
    }

    pub fn set_extra_size_ref(&mut self, extra_size: f32) -> &mut Self {
        self.extra_size = extra_size;
        self
    }
    pub fn set_extra_size(mut self, extra_size: f32) -> Self {
        self.set_extra_size_ref(extra_size);
        self
    }

    pub fn set_extra_spacing_ref(&mut self, extra_spacing: f32) -> &mut Self {
        self.extra_spacing = extra_spacing;
        self
    }
    pub fn set_extra_spacing(mut self, extra_spacing: f32) -> Self {
        self.set_extra_spacing_ref(extra_spacing);
        self
    }

    pub fn join(&mut self, string: ColoredString) -> &mut Self {
        self.lines.extend(string.lines);
        self
    }

    pub fn button(&self, ui: &mut Ui) -> Response {
        Button::new(self.widget(1.0, ui)).wrap(false).ui(ui)
    }
    pub fn label(&self, ui: &mut Ui) -> Response {
        self.as_label(ui).selectable(false).wrap(false).ui(ui)
    }
    pub fn label_alpha(&self, a: f32, ui: &mut Ui) -> Response {
        self.as_label_alpha(a, ui).ui(ui)
    }
    pub fn as_label(&self, ui: &mut Ui) -> Label {
        self.as_label_alpha(1.0, ui)
    }
    pub fn as_label_alpha(&self, a: f32, ui: &mut Ui) -> Label {
        Label::new(self.widget(a, ui))
    }

    pub fn widget(&self, alpha: f32, ui: &mut Ui) -> WidgetText {
        let mut job = LayoutJob::default();
        for (s, color, style) in self.lines.iter() {
            let color = color.unwrap_or(light_gray()).gamma_multiply(alpha);
            let mut font_id = style.get_font(ui.style());
            font_id.size += self.extra_size;
            job.append(
                s,
                0.0,
                TextFormat {
                    color,
                    font_id,
                    extra_letter_spacing: self.extra_spacing,
                    ..default()
                },
            );
        }
        WidgetText::LayoutJob(job)
    }

    pub fn rich_text(&self, ui: &mut Ui) -> RichText {
        let (color, mut font) = if let Some((_, c, f)) = self.lines.get(0) {
            (c.unwrap_or(light_gray()), f.get_font(ui.style()))
        } else {
            (light_gray(), default())
        };
        font.size += self.extra_size;
        RichText::new(self.to_string())
            .color(color)
            .font(font)
            .extra_letter_spacing(self.extra_spacing)
    }

    pub fn inject_definitions(mut self, world: &World) -> Self {
        let mut result: Vec<(String, Option<Color32>, ColoredStringStyle)> = default();
        for (s, color, style) in self.lines.drain(..) {
            for (s, bracketed) in s.split_by_brackets(("[", "]")) {
                if bracketed {
                    let color = if let Ok(color) = Pools::get_color_by_name(&s, world) {
                        color.c32()
                    } else {
                        error!("Failed to find house for ability {s}");
                        light_gray()
                    };
                    result.push((s, Some(color), ColoredStringStyle::Bold));
                } else {
                    result.push((s, color, style));
                }
            }
        }
        self.lines = result;
        self
    }

    pub fn inject_trigger(mut self, state: &VarState) -> Self {
        if let Ok(trigger) = state.get_string(VarName::TriggerDescription) {
            self.lines = self
                .lines
                .drain(..)
                .flat_map(|(line, color, style)| match line.find("%trigger") {
                    Some(_) => line
                        .replace("%trigger", &format!("<{trigger}>"))
                        .split_by_brackets(("<", ">"))
                        .into_iter()
                        .map(|(line, t)| match t {
                            true => (line, Some(white()), style),
                            false => (line, color, style),
                        })
                        .collect_vec(),
                    None => vec![(line, color, style)],
                })
                .collect_vec();
        }
        if let Ok(effect) = state.get_string(VarName::EffectDescription) {
            for (line, _, _) in self.lines.iter_mut() {
                *line = line.replace("%effect", &effect);
            }
        }
        if let Ok(target) = state.get_string(VarName::TargetDescription) {
            for (line, _, _) in self.lines.iter_mut() {
                *line = line.replace("%target", &target);
            }
        }
        self
    }

    pub fn inject_vars(mut self, state: &VarState) -> Self {
        let t = GameTimer::get().play_head();
        let mut result: Vec<(String, Option<Color32>, ColoredStringStyle)> = default();
        for (s, color, style) in self.lines.drain(..) {
            for (mut s, bracketed) in s.split_by_brackets(("{", "}")) {
                if bracketed {
                    if let Some((var, s)) = s.split_once("|") {
                        if VarName::from_str(var)
                            .is_ok_and(|var| state.get_bool_at(var, t).unwrap_or_default())
                        {
                            result.push((s.to_owned(), Some(white()), style));
                        }
                        continue;
                    }
                    if let Ok(var) = VarName::from_str(&s) {
                        if let Ok(value) = state.get_string_at(var, t) {
                            s = value;
                        }
                    }
                    result.push((s, Some(white()), style));
                } else {
                    result.push((s, color, style));
                }
            }
        }
        self.lines = result;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() || self.lines.iter().all(|(s, _, _)| s.is_empty())
    }
    pub fn take(&mut self) -> Self {
        mem::take(self)
    }
}

impl From<&str> for ColoredString {
    fn from(value: &str) -> Self {
        Self {
            lines: vec![(value.to_owned(), None, default())],
            extra_size: 0.0,
            extra_spacing: 0.0,
        }
    }
}

impl ToString for ColoredString {
    fn to_string(&self) -> String {
        self.lines.iter().map(|(s, _, _)| s).join(" ")
    }
}

pub trait ToColoredString {
    fn to_colored(self) -> ColoredString;
    fn add_color(self, color: Color32) -> ColoredString;
}

impl<'a> ToColoredString for &'a str {
    fn to_colored(self) -> ColoredString {
        ColoredString::simple(self.to_owned())
    }

    fn add_color(self, color: Color32) -> ColoredString {
        ColoredString::new(self.to_owned(), color)
    }
}
