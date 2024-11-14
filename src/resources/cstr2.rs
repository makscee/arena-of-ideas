use super::*;

#[derive(Default)]
struct StyleState {
    stack: Vec<CstrStyle>,
}

enum CstrStyle {
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
    fn get_font(&self, style: &Style) -> Option<FontId> {
        match self {
            Self::Small => Some(TextStyle::Small),
            Self::Bold => Some(TextStyle::Name("Bold".into())),
            Self::Heading => Some(TextStyle::Heading),
            Self::Heading2 => Some(TextStyle::Name("Heading2".into())),
            _ => None,
        }
        .map(|s| s.resolve(style))
    }
}

impl CstrStyle {
    fn from_str(value: &str) -> Result<Self, String> {
        match value {
            "b" => Ok(Self::Bold),
            "s" => Ok(Self::Small),
            "h" => Ok(Self::Heading),
            "h2" => Ok(Self::Heading2),

            "red" => Ok(Self::Color(RED)),
            "green" => Ok(Self::Color(GREEN)),
            "vd" => Ok(Self::Color(VISIBLE_DARK)),
            "vl" => Ok(Self::Color(VISIBLE_LIGHT)),
            "vb" => Ok(Self::Color(VISIBLE_BRIGHT)),
            _ => Err(format!("Unknown style token {value}")),
        }
    }
}

impl StyleState {
    fn append(&self, text: &mut String, job: &mut LayoutJob, style: &Style) {
        let color = self
            .stack
            .iter()
            .rev()
            .find_map(|s| s.get_color())
            .unwrap_or_else(|| style.visuals.widgets.noninteractive.fg_stroke.color);
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
    fn push(&mut self, token: &str) {
        match CstrStyle::from_str(token) {
            Ok(v) => self.stack.push(v),
            Err(e) => error!("Failed to parse token: {e}"),
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

enum ParseState {
    Token,
    Var,
    Text,
}

pub fn cstr_parse(s: &str, get_var: fn(VarName) -> String, ui: &mut Ui) -> WidgetText {
    let mut job = LayoutJob::default();
    cstr_parse_into_job(s, get_var, &mut job, ui);
    WidgetText::LayoutJob(job)
}
fn cstr_parse_into_job(s: &str, get_var: fn(VarName) -> String, job: &mut LayoutJob, ui: &mut Ui) {
    let mut cur = String::new();
    let mut style_state: StyleState = default();
    let mut parse_state = ParseState::Text;
    for c in s.chars() {
        match c {
            '[' => {
                style_state.append(&mut cur, job, ui.style());
                parse_state = ParseState::Token;
            }
            ']' => {
                style_state.append(&mut cur, job, ui.style());
                style_state.pop();
            }
            '$' => {
                style_state.append(&mut cur, job, ui.style());
                parse_state = ParseState::Var;
            }
            ' ' => match parse_state {
                ParseState::Token => {
                    style_state.push(&cur);
                    cur.clear();
                    parse_state = ParseState::Text;
                }
                ParseState::Var => match VarName::from_str(&cur) {
                    Ok(v) => {
                        let var_str = get_var(v);
                        let s = format!("[vd [s {v}:]][vb {var_str}] ");
                        cstr_parse_into_job(&s, |_| default(), job, ui);
                        cur.clear();
                    }
                    Err(e) => error!("Failed to parse var {cur}: {e}"),
                },
                ParseState::Text => cur.push(c),
            },
            _ => cur.push(c),
        }
    }
    if !cur.is_empty() {
        style_state.append(&mut cur, job, ui.style());
    }
}
