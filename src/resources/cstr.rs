use super::*;

#[derive(Default, Clone)]
pub struct Cstr {
    subs: Vec<CstrSub>,
}

#[derive(Clone)]
struct CstrSub {
    text: String,
    color: Option<Color32>,
    style: CstrStyle,
}

#[derive(Clone, Copy, Debug, Default)]
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
                    text.custom_color(color)
                },
            )
            .join(" ")
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
                text,
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

    pub fn take(&mut self) -> Self {
        mem::take(self)
    }
}

impl ToString for Cstr {
    fn to_string(&self) -> String {
        self.subs.iter().map(|s| &s.text).join(" ")
    }
}

pub trait ToCstr {
    fn cstr(self) -> Cstr;
    fn cstr_c(self, color: Color32) -> Cstr;
    fn cstr_cs(self, color: Color32, style: CstrStyle) -> Cstr;
}

impl<'a> ToCstr for &'a str {
    fn cstr(self) -> Cstr {
        Cstr {
            subs: vec![CstrSub {
                text: self.to_owned(),
                color: None,
                style: default(),
            }],
        }
    }
    fn cstr_c(self, color: Color32) -> Cstr {
        Cstr {
            subs: vec![CstrSub {
                text: self.to_owned(),
                color: Some(color),
                style: default(),
            }],
        }
    }
    fn cstr_cs(self, color: Color32, style: CstrStyle) -> Cstr {
        Cstr {
            subs: vec![CstrSub {
                text: self.to_owned(),
                color: Some(color),
                style,
            }],
        }
    }
}
