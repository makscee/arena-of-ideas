use super::*;

#[derive(Clone, Debug, Default)]
pub struct ColoredString {
    pub lines: Vec<(String, Option<Color32>)>,
}

impl ColoredString {
    pub fn simple(text: String) -> Self {
        Self {
            lines: [(text, None)].into(),
        }
    }

    pub fn new(text: String, color: Color32) -> Self {
        Self {
            lines: vec![(text, Some(color))],
        }
    }

    pub fn push(&mut self, text: String, color: Color32) -> &mut Self {
        self.lines.push((text, Some(color)));
        self
    }

    pub fn widget(&self) -> WidgetText {
        let mut job = LayoutJob::default();
        for (s, color) in self.lines.iter() {
            let color = color.unwrap_or(light_gray());
            job.append(&s, 0.0, TextFormat { color, ..default() });
        }
        WidgetText::LayoutJob(job)
    }

    pub fn rich_text(&self) -> RichText {
        let color = self
            .lines
            .get(0)
            .and_then(|(_, c)| *c)
            .unwrap_or(light_gray());
        RichText::new(self.to_string()).color(color)
    }

    pub fn inject_definitions(mut self, world: &World) -> Self {
        let mut result: Vec<(String, Option<Color32>)> = default();
        for (s, color) in self.lines.drain(..) {
            if color.is_some() {
                result.push((s, color));
                continue;
            }
            for (s, bracketed) in str_extract_brackets(&s, ("[", "]")) {
                if bracketed {
                    let color = if let Some(house) = Pools::get_ability_house(&s, world) {
                        house.color.clone().into()
                    } else {
                        error!("Failed to find house for ability {s}");
                        light_gray()
                    };
                    result.push((s, Some(color)));
                } else {
                    result.push((s, None));
                }
            }
        }
        self.lines = result;
        self
    }

    pub fn inject_vars(mut self, state: &VarState) -> Self {
        let mut result: Vec<(String, Option<Color32>)> = default();
        for (s, color) in self.lines.drain(..) {
            if color.is_some() {
                result.push((s, color));
                continue;
            }
            for (mut s, bracketed) in str_extract_brackets(&s, ("{", "}")) {
                if bracketed {
                    if let Ok(var) = VarName::from_str(&s) {
                        if let Ok(value) = state.get_string(var) {
                            s = value;
                        }
                    }
                    result.push((s, Some(white())));
                } else {
                    result.push((s, None));
                }
            }
        }
        self.lines = result;
        self
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() || self.lines.iter().all(|(s, _)| s.is_empty())
    }
}

impl From<&str> for ColoredString {
    fn from(value: &str) -> Self {
        Self {
            lines: vec![(value.to_owned(), None)],
        }
    }
}

impl ToString for ColoredString {
    fn to_string(&self) -> String {
        self.lines.iter().map(|(s, _)| s).join(" ")
    }
}

fn str_extract_brackets(mut source: &str, pattern: (&str, &str)) -> Vec<(String, bool)> {
    let mut lines: Vec<(String, bool)> = default();
    while let Some(opening) = source.find(pattern.0) {
        let left = &source[..opening];
        let closing = source.find(pattern.1).unwrap();
        let mid = &source[opening + 1..closing];
        lines.push((left.to_owned(), false));
        lines.push((mid.to_owned(), true));
        source = &source[closing + 1..];
    }
    lines.push((source.to_owned(), false));
    lines
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
