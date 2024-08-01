use std::str::FromStr;

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
            SubText::VarText(_, _) => default(),
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
    pub fn push_wrapped(&mut self, mut cstr: Cstr, brackets: (char, char)) -> &mut Self {
        self.push(
            cstr.wrap((brackets.0.to_string().cstr(), brackets.1.to_string().cstr()))
                .take(),
        )
    }
    pub fn push_wrapped_curly(&mut self, mut cstr: Cstr) -> &mut Self {
        self.push(cstr.wrap(("(".cstr(), ")".cstr())).take())
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
                    let color = color.unwrap_or(VISIBLE_DARK);
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
    pub fn replace_absent_color(&mut self, color: Color32) -> &mut Self {
        for sub in self.subs.iter_mut() {
            if sub.color.is_none() {
                sub.color = Some(color);
            }
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

    pub fn join(&mut self, s: &Cstr) -> &mut Self {
        let subs = mem::take(&mut self.subs);
        let len = subs.len();
        for (i, sub) in subs.into_iter().enumerate() {
            self.subs.push(sub);
            if i == len - 1 {
                break;
            }
            for sub in &s.subs {
                self.subs.push(sub.clone());
            }
        }
        self
    }
    pub fn join_char(&mut self, c: char) -> &mut Self {
        self.join(&c.to_string().cstr())
    }
    pub fn join_vec(v: Vec<Self>) -> Self {
        Self {
            subs: v.into_iter().flat_map(|v| v.subs).collect_vec(),
        }
    }

    pub fn button(self, ui: &mut Ui) -> Response {
        Button::click(self.to_string()).cstr(self).ui(ui)
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
            let ui_style = ui.style();
            let color = color
                .unwrap_or(ui_style.visuals.widgets.noninteractive.fg_stroke.color)
                .gamma_multiply(alpha);
            let font_id = style.get_font(ui_style);
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

    pub fn inject_context(&mut self, context: &Context, world: &World) -> &mut Self {
        for mut sub in self.subs.drain(..).collect_vec() {
            match &sub.text {
                SubText::String(_) => self.subs.push(sub),
                SubText::Var(var) => {
                    sub.text = match context.get_string(*var, world) {
                        Ok(v) => v,
                        Err(e) => format!("err: {e}"),
                    }
                    .into();
                    self.subs.push(sub);
                }
                SubText::VarText(var, text) => {
                    if context.get_bool(*var, world).unwrap_or_default() {
                        let cs = Self::parse(text);
                        self.subs.extend(cs.subs.into_iter());
                    }
                }
            };
        }
        self
    }

    pub fn inject_ability_state(&mut self, ability: &str, context: &Context) -> &mut Self {
        for mut sub in self.subs.drain(..).collect_vec() {
            match &sub.text {
                SubText::String(_) => self.subs.push(sub),
                SubText::Var(var) => {
                    if let Ok(value) = context.get_ability_var(ability, *var) {
                        sub.text = SubText::String(value.get_string().unwrap())
                    }
                    self.subs.push(sub);
                }
                SubText::VarText(var, text) => {
                    if context
                        .get_ability_var(ability, *var)
                        .is_ok_and(|v| v.get_bool().unwrap_or_default())
                    {
                        let cs = Self::parse(text);
                        self.subs.extend(cs.subs.into_iter());
                    }
                }
            };
        }
        self
    }
    fn parse_var(s: &str) -> CstrSub {
        if let Some((s, text)) = s.split_once('|') {
            let var = VarName::from_str(s).unwrap();
            CstrSub {
                text: SubText::VarText(var, text.into()),
                color: Some(VISIBLE_BRIGHT),
                style: default(),
            }
        } else {
            match VarName::from_str(s) {
                Ok(var) => {
                    let mut var: CstrSub = var.into();
                    var.color = Some(VISIBLE_BRIGHT);
                    var
                }
                Err(_) => CstrSub {
                    text: s.into(),
                    color: Some(VISIBLE_BRIGHT),
                    style: CstrStyle::Bold,
                },
            }
        }
    }
    fn parse_definition(s: &str) -> CstrSub {
        let color = name_color(s);
        CstrSub {
            text: s.into(),
            color: Some(color),
            style: CstrStyle::Bold,
        }
    }
    pub fn parse(s: &str) -> Self {
        let mut cs = Cstr::default();
        for (s, bracketed) in s.split_by_brackets('{', '}') {
            if bracketed {
                cs.subs.push(Self::parse_var(&s));
            } else {
                cs.subs.push(s.into());
            }
        }
        for sub in cs.subs.drain(..).collect_vec() {
            match sub.text {
                SubText::String(text) => {
                    for (s, bracketed) in text.split_by_brackets('[', ']') {
                        if bracketed {
                            cs.subs.push(Self::parse_definition(&s));
                        } else {
                            cs.subs.push(CstrSub {
                                text: s.into(),
                                color: sub.color,
                                style: sub.style,
                            });
                        }
                    }
                }
                _ => cs.subs.push(sub),
            }
        }

        cs
    }

    pub fn take(&mut self) -> Self {
        mem::take(self)
    }
}

impl std::fmt::Display for Cstr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_colored())
    }
}

pub trait ToCstr: Sized {
    fn cstr(&self) -> Cstr;
    fn cstr_c(&self, color: Color32) -> Cstr {
        self.cstr().color(color).take()
    }
    fn cstr_cs(&self, color: Color32, style: CstrStyle) -> Cstr {
        self.cstr().color(color).style(style).take()
    }
}

impl<'a> ToCstr for &'a str {
    fn cstr(&self) -> Cstr {
        Cstr {
            subs: vec![CstrSub {
                text: (*self).into(),
                color: None,
                style: default(),
            }],
        }
    }
    fn cstr_c(&self, color: Color32) -> Cstr {
        Cstr {
            subs: vec![CstrSub {
                text: (*self).into(),
                color: Some(color),
                style: default(),
            }],
        }
    }
    fn cstr_cs(&self, color: Color32, style: CstrStyle) -> Cstr {
        Cstr {
            subs: vec![CstrSub {
                text: (*self).into(),
                color: Some(color),
                style,
            }],
        }
    }
}

impl ToCstr for String {
    fn cstr(&self) -> Cstr {
        Cstr {
            subs: vec![CstrSub {
                text: self.clone().into(),
                color: None,
                style: default(),
            }],
        }
    }
}
impl ToCstr for VarName {
    fn cstr(&self) -> Cstr {
        Cstr {
            subs: vec![CstrSub {
                text: (*self).into(),
                color: None,
                style: default(),
            }],
        }
    }
}
impl ToCstr for TBaseUnit {
    fn cstr(&self) -> Cstr {
        let color = name_color(&self.house);
        self.name.cstr_c(color)
    }
}
impl ToCstr for FusedUnit {
    fn cstr(&self) -> Cstr {
        self.cstr_limit(3)
    }
}
impl ToCstr for TTeam {
    fn cstr(&self) -> Cstr {
        self.units
            .iter()
            .map(|u| u.cstr_limit(1).push(" ".cstr()).take())
            .collect_vec()
            .into()
    }
}
impl ToCstr for TUser {
    fn cstr(&self) -> Cstr {
        self.name.cstr_cs(VISIBLE_LIGHT, CstrStyle::Bold)
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
impl From<String> for CstrSub {
    fn from(value: String) -> Self {
        Self {
            text: value.into(),
            color: default(),
            style: default(),
        }
    }
}
impl From<Vec<Cstr>> for Cstr {
    fn from(value: Vec<Cstr>) -> Self {
        Self {
            subs: value.into_iter().flat_map(|v| v.subs).collect_vec(),
        }
    }
}

impl FusedUnit {
    pub fn cstr_limit(&self, max_chars: usize) -> Cstr {
        UnitPlugin::name_from_bases(self.bases.iter().map(|s| s.as_str()).collect(), max_chars)
    }
}
