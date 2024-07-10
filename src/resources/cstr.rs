use std::str::FromStr;

use regex::Regex;

use super::*;

#[derive(Default, Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Cstr {
    subs: Vec<CstrSub>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
struct CstrSub {
    text: SubText,
    color: Option<Color32>,
    style: CstrStyle,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq)]
enum SubText {
    String(String),
    Var(VarName),
    VarText(VarName, String),
}

impl SubText {
    fn str(&self) -> &str {
        match self {
            SubText::String(s) => s,
            SubText::Var(var) => var.as_ref(),
            SubText::VarText(var, _) => default(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub enum CstrStyle {
    #[default]
    Normal,
    Small,
    Bold,
    Heading,
    Heading2,
}

impl CstrStyle {
    fn get_font(&self, style: &Style) -> FontId {
        match self {
            Self::Normal => TextStyle::Body,
            Self::Small => TextStyle::Small,
            Self::Bold => TextStyle::Name("Bold".into()),
            Self::Heading => TextStyle::Heading,
            Self::Heading2 => TextStyle::Name("Heading2".into()),
        }
        .resolve(style)
    }
}

impl Cstr {
    pub fn push(&mut self, cstr: Cstr) -> &mut Self {
        self.subs.extend(cstr.subs.into_iter());
        self
    }
    fn to_colored(&self) -> String {
        self.subs
            .iter()
            .map(
                |CstrSub {
                     text,
                     color,
                     style: _,
                 }| {
                    let color = color.unwrap_or(LIGHT_GRAY);
                    let color = CustomColor {
                        r: color.r(),
                        g: color.g(),
                        b: color.b(),
                    };
                    text.str().custom_color(color)
                },
            )
            .join("")
    }
    pub fn print(&self) {
        println!("{}", self.to_colored())
    }
    pub fn info(&self) {
        info!("{}", self.to_colored())
    }
    pub fn debug(&self) {
        debug!("{}", self.to_colored())
    }

    pub fn bold(&mut self) -> &mut Self {
        self.subs.iter_mut().for_each(
            |CstrSub {
                 text: _,
                 color: _,
                 style,
             }| *style = CstrStyle::Bold,
        );
        self
    }
    pub fn color(&mut self, color: Color32) -> &mut Self {
        for sub in self.subs.iter_mut() {
            sub.color = Some(color);
        }
        self
    }
    pub fn style(&mut self, style: CstrStyle) -> &mut Self {
        for sub in self.subs.iter_mut() {
            sub.style = style;
        }
        self
    }
    pub fn wrap(&mut self, mut cs: (Cstr, Cstr)) -> &mut Self {
        let mut subs = cs.0.subs;
        subs.append(&mut self.subs);
        subs.append(&mut cs.1.subs);
        self.subs = subs;
        self
    }

    pub fn join(&mut self, char: &Cstr) -> &mut Self {
        let subs = mem::take(&mut self.subs);
        let len = subs.len();
        for (i, sub) in subs.into_iter().enumerate() {
            self.subs.push(sub);
            if i == len - 1 {
                break;
            }
            for sub in &char.subs {
                self.subs.push(sub.clone());
            }
        }
        self
    }
    pub fn join_vec(v: Vec<Self>) -> Self {
        Self {
            subs: v.into_iter().flat_map(|v| v.subs).collect_vec(),
        }
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
        for CstrSub { text, color, style } in self.subs.iter() {
            let color = color.unwrap_or(LIGHT_GRAY).gamma_multiply(alpha);
            let font_id = style.get_font(ui.style());
            job.append(
                text.str(),
                0.0,
                TextFormat {
                    color,
                    font_id,
                    ..default()
                },
            );
        }
        WidgetText::LayoutJob(job)
    }

    pub fn inject_state(&mut self, state: &VarState, t: f32) -> &mut Self {
        for sub in self.subs.iter_mut() {
            match &sub.text {
                SubText::String(_) => continue,
                SubText::Var(var) => {
                    sub.text = match state.get_string_at(*var, t) {
                        Ok(v) => v,
                        Err(e) => format!("err: {e}"),
                    }
                    .into();
                }
                SubText::VarText(var, text) => {
                    sub.text = match state.get_bool_at(*var, t) {
                        Ok(v) => {
                            if v {
                                text.clone()
                            } else {
                                default()
                            }
                        }
                        Err(_) => default(),
                    }
                    .into();
                }
            };
        }
        self
    }

    pub fn inject_ability_state(
        &mut self,
        ability: &str,
        faction: Faction,
        t: f32,
        world: &World,
    ) -> &mut Self {
        let Some(m) = GameAssets::get(world).ability_defaults.get(ability) else {
            return self;
        };
        let ability_state = TeamPlugin::get_ability_state(ability, faction, world);
        let get_value = |var: &VarName| {
            ability_state
                .and_then(|s| s.get_value_at(*var, t).ok())
                .or_else(|| m.get(var).cloned())
        };
        for sub in &mut self.subs {
            match &sub.text {
                SubText::String(_) => {}
                SubText::Var(var) => {
                    if let Some(value) = get_value(var) {
                        sub.text = SubText::String(value.get_string().unwrap())
                    }
                }
                SubText::VarText(var, text) => {
                    if let Some(value) = get_value(var) {
                        if value.get_bool().unwrap() {
                            sub.text = SubText::String(text.clone())
                        }
                    }
                }
            };
        }
        self
    }
    pub fn parse(mut s: &str) -> Self {
        let mut cs = Cstr::default();
        let mut cut = 0;
        for ele in EXTRACT_BRACKETED.captures_iter(s) {
            let range = ele.get(0).unwrap().range();
            let (a, _) = s.split_at(range.start - cut);
            cs.push(a.cstr());
            let (a, b) = s.split_at(range.end - cut);
            s = b;
            cut += a.len();
            let var_str = ele.get(1).unwrap().as_str();
            if let Some((var_str, text)) = var_str.split_once('|') {
                let var = VarName::from_str(var_str).unwrap();
                cs.subs.push(CstrSub {
                    text: SubText::VarText(var, text.into()),
                    color: Some(WHITE),
                    style: default(),
                });
            } else {
                match VarName::from_str(var_str) {
                    Ok(var) => {
                        let mut var: CstrSub = var.into();
                        var.color = Some(WHITE);
                        cs.subs.push(var);
                    }
                    Err(_) => {
                        cs.push(var_str.cstr_c(WHITE));
                    }
                };
            }
        }
        if !s.is_empty() {
            cs.push(s.cstr());
        }
        cs
    }

    pub fn take(&mut self) -> Self {
        mem::take(self)
    }
}

lazy_static! {
    static ref EXTRACT_BRACKETED: Regex = Regex::new(r"\{(.*?)\}").unwrap();
}

impl ToString for Cstr {
    fn to_string(&self) -> String {
        self.subs.iter().map(|s| s.text.str()).join(" ")
    }
}

pub trait ToCstr: Sized {
    fn cstr(self) -> Cstr;
    fn cstr_c(self, color: Color32) -> Cstr {
        self.cstr().color(color).take()
    }
    fn cstr_cs(self, color: Color32, style: CstrStyle) -> Cstr {
        self.cstr().color(color).style(style).take()
    }
}

impl<'a> ToCstr for &'a str {
    fn cstr(self) -> Cstr {
        Cstr {
            subs: vec![CstrSub {
                text: self.into(),
                color: None,
                style: default(),
            }],
        }
    }
    fn cstr_c(self, color: Color32) -> Cstr {
        Cstr {
            subs: vec![CstrSub {
                text: self.into(),
                color: Some(color),
                style: default(),
            }],
        }
    }
    fn cstr_cs(self, color: Color32, style: CstrStyle) -> Cstr {
        Cstr {
            subs: vec![CstrSub {
                text: self.into(),
                color: Some(color),
                style,
            }],
        }
    }
}

impl From<&str> for SubText {
    fn from(value: &str) -> Self {
        SubText::String(value.into())
    }
}
impl From<String> for SubText {
    fn from(value: String) -> Self {
        SubText::String(value)
    }
}
impl From<VarName> for SubText {
    fn from(value: VarName) -> Self {
        SubText::Var(value)
    }
}

impl From<VarName> for CstrSub {
    fn from(value: VarName) -> Self {
        Self {
            text: value.into(),
            color: default(),
            style: default(),
        }
    }
}
impl From<&str> for CstrSub {
    fn from(value: &str) -> Self {
        Self {
            text: value.into(),
            color: default(),
            style: default(),
        }
    }
}
