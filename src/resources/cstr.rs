use colored::{Colorize, CustomColor};

use super::*;

pub struct Cstr {
    subs: Vec<CstrSub>,
}

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

impl Cstr {
    pub fn push(&mut self, cstr: Cstr) -> &mut Self {
        self.subs.extend(cstr.subs.into_iter());
        self
    }
    fn to_colored(&self) -> String {
        self.subs
            .iter()
            .map(|CstrSub { text, color, style }| {
                let color = color.unwrap_or(LIGHT_GRAY);
                let color = CustomColor {
                    r: color.r(),
                    g: color.g(),
                    b: color.b(),
                };
                text.custom_color(color)
            })
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
