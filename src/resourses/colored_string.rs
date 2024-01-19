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

    pub fn join(&mut self, string: ColoredString) -> &mut Self {
        self.lines.extend(string.lines.into_iter());
        self
    }

    pub fn widget(&self) -> WidgetText {
        self.widget_with_font(None)
    }

    pub fn widget_with_font(&self, font_id: Option<FontId>) -> WidgetText {
        let mut job = LayoutJob::default();
        let font_id = font_id.unwrap_or_default();
        for (s, color) in self.lines.iter() {
            let color = color.unwrap_or(light_gray());
            job.append(
                s,
                0.0,
                TextFormat {
                    color,
                    font_id: font_id.clone(),
                    ..default()
                },
            );
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
            for (s, bracketed) in s.split_by_brackets(("[", "]")) {
                if bracketed {
                    let color = if let Some(house) =
                        Pools::get_ability_house(&s, world).or(Pools::get_status_house(&s, world))
                    {
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
            for (mut s, bracketed) in s.split_by_brackets(("{", "}")) {
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
    pub fn take(&mut self) -> Self {
        mem::take(self)
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
