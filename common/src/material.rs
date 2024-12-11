use bevy::math::vec2;
use bevy_egui::egui::{DragValue, Ui, Widget};

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RMaterial {
    pub t: MaterialType,
    #[serde(default)]
    pub count: u32,
    #[serde(default)]
    pub modifiers: Vec<RModifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumIter, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum RModifier {
    Color(Expression),
    Offset(Expression),
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumIter, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum MaterialType {
    Shape {
        shape: Shape,
        #[serde(default)]
        modifiers: Vec<ShapeModifier>,
    },
    Text {
        text: Expression,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumIter, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum Shape {
    Rectangle { size: Expression },
    Circle { radius: Expression },
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumIter, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum ShapeModifier {
    Scale(Expression),
    Rotation(Expression),
    Color(Expression),
    Hollow(Expression),
    Thickness(Expression),
    Roundness(Expression),
    Alpha(Expression),
}

impl Default for RModifier {
    fn default() -> Self {
        RModifier::Offset(Expression::V2(0.0, 0.0))
    }
}
impl Default for Shape {
    fn default() -> Self {
        Shape::Rectangle {
            size: Expression::V(vec2(1.0, 1.0).into()),
        }
    }
}
impl Default for MaterialType {
    fn default() -> Self {
        Self::Shape {
            shape: default(),
            modifiers: default(),
        }
    }
}
impl Default for ShapeModifier {
    fn default() -> Self {
        Self::Rotation(Expression::Zero)
    }
}

impl ToCstr for RMaterial {
    fn cstr(&self) -> Cstr {
        format!(
            "[b [vb x{}]]\n{}\n({})",
            self.count,
            self.t.cstr(),
            self.modifiers.iter().map(|x| x.cstr()).join(", ")
        )
    }
    fn cstr_expanded(&self) -> Cstr {
        format!(
            "[b [vb x{}]]\n{}\n({})",
            self.count,
            self.t.cstr_expanded(),
            self.modifiers.iter().map(|x| x.cstr_expanded()).join(", ")
        )
    }
}
impl ToCstr for RModifier {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr_c(CYAN)
    }
    fn cstr_expanded(&self) -> Cstr {
        let inner = match self {
            RModifier::Color(x) | RModifier::Offset(x) => x.cstr_expanded(),
        };
        format!("{}({inner})", self.cstr())
    }
}
impl ToCstr for MaterialType {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr_c(CYAN)
    }
    fn cstr_expanded(&self) -> Cstr {
        let inner = match self {
            MaterialType::Shape { shape, modifiers } => format!(
                "{}, ({})",
                shape.cstr_expanded(),
                modifiers.into_iter().map(|m| m.cstr_expanded()).join(", ")
            ),
            MaterialType::Text { text } => text.cstr_expanded(),
        };
        format!("{}({inner})", self.cstr())
    }
}
impl ToCstr for ShapeModifier {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr_c(CYAN)
    }
    fn cstr_expanded(&self) -> Cstr {
        let inner = match self {
            ShapeModifier::Rotation(x)
            | ShapeModifier::Scale(x)
            | ShapeModifier::Color(x)
            | ShapeModifier::Hollow(x)
            | ShapeModifier::Thickness(x)
            | ShapeModifier::Roundness(x)
            | ShapeModifier::Alpha(x) => x.cstr_expanded(),
        };
        format!("{}({inner})", self.cstr())
    }
}
impl ToCstr for Shape {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr_c(CYAN)
    }
    fn cstr_expanded(&self) -> Cstr {
        let inner = match self {
            Shape::Rectangle { size: x } | Shape::Circle { radius: x } => x.cstr_expanded(),
        };
        format!("{}({inner})", self.cstr())
    }
}

impl Show for RMaterial {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
        format!("{}{}", prefix.unwrap_or_default(), self.cstr_expanded()).label_w(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        if let Some(prefix) = prefix {
            prefix.cstr().label(ui);
        }
        let mut changed = self.t.show_mut(Some("type:"), ui);
        changed |= DragValue::new(&mut self.count)
            .prefix("count:")
            .ui(ui)
            .changed();
        for m in &mut self.modifiers {
            changed |= m.show_mut(None, ui);
        }
        if "+"
            .cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold)
            .button(ui)
            .clicked()
        {
            self.modifiers.push(default());
        }
        changed
    }
}
impl Show for RModifier {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
        format!("{}{}", prefix.unwrap_or_default(), self.cstr_expanded()).label_w(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        Selector::new(prefix.unwrap_or_default()).ui_enum(self, ui)
            || match self {
                RModifier::Color(x) | RModifier::Offset(x) => x.show_mut(None, ui),
            }
    }
}
impl Show for MaterialType {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
        format!("{}{}", prefix.unwrap_or_default(), self.cstr_expanded()).label_w(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        Selector::new(prefix.unwrap_or_default()).ui_enum(self, ui)
            || match self {
                MaterialType::Shape { shape, modifiers } => {
                    let mut c = shape.show_mut(Some("shape:"), ui);
                    for (i, m) in modifiers.iter_mut().enumerate() {
                        ui.push_id(i, |ui| {
                            c |= m.show_mut(None, ui);
                        });
                    }
                    c
                }
                MaterialType::Text { text } => todo!(),
            }
    }
}
impl Show for ShapeModifier {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
        format!("{}{}", prefix.unwrap_or_default(), self.cstr_expanded()).label_w(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        Selector::new(prefix.unwrap_or_default()).ui_enum(self, ui)
            || match self {
                ShapeModifier::Scale(x)
                | ShapeModifier::Rotation(x)
                | ShapeModifier::Color(x)
                | ShapeModifier::Hollow(x)
                | ShapeModifier::Thickness(x)
                | ShapeModifier::Roundness(x)
                | ShapeModifier::Alpha(x) => x.show_mut(None, ui),
            }
    }
}
impl Show for Shape {
    fn show(&self, prefix: Option<&str>, ui: &mut Ui) {
        format!("{}{}", prefix.unwrap_or_default(), self.cstr_expanded()).label_w(ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        Selector::new(prefix.unwrap_or_default()).ui_enum(self, ui)
            || match self {
                Shape::Rectangle { size } => size.show_mut(Some("size:"), ui),
                Shape::Circle { radius } => radius.show_mut(Some("radius:"), ui),
            }
    }
}
