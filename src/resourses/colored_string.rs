use bevy_egui::egui::Style;

use super::*;

#[derive(Clone, Debug, Default)]
pub struct ColoredString {
    pub lines: Vec<(String, Option<Color32>, ColoredStringStyle)>,
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
        }
    }

    pub fn new(text: String, color: Color32) -> Self {
        Self {
            lines: vec![(text, Some(color), default())],
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
        self.lines.iter_mut().for_each(|(_, _, s)| *s = style);
        self
    }

    pub fn join(&mut self, string: ColoredString) -> &mut Self {
        self.lines.extend(string.lines);
        self
    }

    pub fn label(&self, ui: &mut Ui) -> Response {
        self.as_label(ui).ui(ui)
    }
    pub fn as_label(&self, ui: &mut Ui) -> Label {
        Label::new(self.widget(ui))
    }

    pub fn widget(&self, ui: &mut Ui) -> WidgetText {
        let mut job = LayoutJob::default();
        for (s, color, style) in self.lines.iter() {
            let color = color.unwrap_or(light_gray());
            job.append(
                s,
                0.0,
                TextFormat {
                    color,
                    font_id: style.get_font(ui.style()),
                    ..default()
                },
            );
        }
        WidgetText::LayoutJob(job)
    }

    pub fn rich_text(&self, ui: &mut Ui) -> RichText {
        let (color, font) = if let Some((_, c, f)) = self.lines.get(0) {
            (c.unwrap_or(light_gray()), f.get_font(ui.style()))
        } else {
            (light_gray(), default())
        };
        RichText::new(self.to_string()).color(color).font(font)
    }

    pub fn inject_definitions(mut self, world: &World) -> Self {
        let mut result: Vec<(String, Option<Color32>, ColoredStringStyle)> = default();
        for (s, color, style) in self.lines.drain(..) {
            // if color.is_some() {
            //     result.push((s, color, style));
            //     continue;
            // }
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
        let mut result: Vec<(String, Option<Color32>, ColoredStringStyle)> = default();
        for (s, color, style) in self.lines.drain(..) {
            // if color.is_some() {
            //     result.push((s, color, style));
            //     continue;
            // }
            for (mut s, bracketed) in s.split_by_brackets(("{", "}")) {
                if bracketed {
                    if let Ok(var) = VarName::from_str(&s) {
                        if let Ok(value) = state.get_string(var) {
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
