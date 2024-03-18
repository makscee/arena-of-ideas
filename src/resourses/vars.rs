use std::{cmp::Ordering, fmt::Display};

use anyhow::anyhow;
use bevy_egui::egui::epaint::util::FloatOrd;
use strum_macros::{Display, EnumString};

use super::*;

#[derive(
    Hash,
    Eq,
    PartialEq,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    Reflect,
    Display,
    EnumString,
    EnumIter,
    Default,
    PartialOrd,
    Ord,
)]
pub enum VarName {
    #[default]
    None,
    Offset,
    Rotation,
    Charges,
    Hp,
    Atk,
    Faction,
    Position,
    Scale,
    Value,
    Size,
    Radius,
    Houses,
    HouseColor1,
    HouseColor2,
    HouseColor3,
    Dmg,
    Name,
    Description,
    EffectDescription,
    TriggerDescription,
    TargetDescription,
    Text,
    Spawn,
    Slot,
    Visible,
    Direction,
    G,
    LastAttacker,
    Color,
    Thickness,
    Curvature,
    Delta,
    T,
    Count,
    Alpha,
    Index,
    StatusIndex,
    Stacks,
    Level,
    Id,
    Caster,
}

#[derive(Serialize, Deserialize, Clone, Debug, Reflect, PartialEq, Default)]
pub enum VarValue {
    #[default]
    None,
    Float(f32),
    Int(i32),
    Vec2(Vec2),
    Bool(bool),
    String(String),
    Faction(Faction),
    Entity(Entity),
    EntityList(Vec<Entity>),
    Color(Color),
}

impl VarValue {
    pub fn get_float(&self) -> Result<f32> {
        match self {
            VarValue::Float(value) => Ok(*value),
            VarValue::Int(value) => Ok(*value as f32),
            VarValue::Bool(value) => Ok(*value as i32 as f32),
            VarValue::None => Ok(0.0),
            _ => Err(anyhow!("Float not supported by {self:?}")),
        }
    }
    pub fn get_int(&self) -> Result<i32> {
        match self {
            VarValue::Int(value) => Ok(*value),
            VarValue::Float(value) => Ok(*value as i32),
            VarValue::Bool(value) => Ok(*value as i32),
            VarValue::None => Ok(0),
            _ => Err(anyhow!("Int not supported by {self:?}")),
        }
    }
    pub fn get_vec2(&self) -> Result<Vec2> {
        match self {
            VarValue::Vec2(value) => Ok(*value),
            VarValue::None => Ok(Vec2::ZERO),
            _ => Err(anyhow!("Vec2 not supported by {self:?}")),
        }
    }
    pub fn get_bool(&self) -> Result<bool> {
        match self {
            VarValue::Bool(value) => Ok(*value),
            VarValue::Int(value) => Ok(*value > 0),
            VarValue::Float(value) => Ok(*value > 0.0),
            VarValue::String(value) => Ok(!value.is_empty()),
            VarValue::None => Ok(false),
            _ => Err(anyhow!("Bool not supported by {self:?}")),
        }
    }
    pub fn get_string(&self) -> Result<String> {
        match self {
            VarValue::String(value) => Ok(value.into()),
            VarValue::Int(value) => Ok(value.to_string()),
            VarValue::Float(value) => Ok(value.to_string()),
            VarValue::Vec2(value) => Ok(value.to_string()),
            VarValue::Bool(value) => Ok(value.to_string()),
            VarValue::Faction(value) => Ok(value.to_string()),
            _ => Err(anyhow!("String not supported by {self:?}")),
        }
    }
    pub fn get_faction(&self) -> Result<Faction> {
        match self {
            VarValue::Faction(value) => Ok(*value),
            _ => Err(anyhow!("Faction not supported by {self:?}")),
        }
    }
    pub fn get_color(&self) -> Result<Color> {
        match self {
            VarValue::Color(value) => Ok(*value),
            _ => Err(anyhow!("Color not supported by {self:?}")),
        }
    }
    pub fn get_entity(&self) -> Result<Entity> {
        match self {
            VarValue::Entity(value) => Ok(*value),
            _ => Err(anyhow!("Entity not supported by {self:?}")),
        }
    }
    pub fn get_entity_list(&self) -> Result<Vec<Entity>> {
        match self {
            VarValue::EntityList(value) => Ok(value.clone()),
            _ => Err(anyhow!("Entity not supported by {self:?}")),
        }
    }

    pub fn sum(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(a + b)),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(VarValue::Int(a + b)),
            (VarValue::Float(a), VarValue::Int(b)) => Ok(VarValue::Float(a + *b as f32)),
            (VarValue::Int(a), VarValue::Float(b)) => Ok(VarValue::Float(b + *a as f32)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a + *b)),
            (VarValue::String(a), VarValue::String(b)) => Ok(VarValue::String(a.to_owned() + b)),
            _ => Err(anyhow!("{a:?} + {b:?} not supported")),
        }
    }

    pub fn sub(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(a - b)),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(VarValue::Int(a - b)),
            (VarValue::Float(a), VarValue::Int(b)) => Ok(VarValue::Float(a - *b as f32)),
            (VarValue::Int(a), VarValue::Float(b)) => Ok(VarValue::Float(*a as f32 - b)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a - *b)),
            _ => Err(anyhow!("{a:?} - {b:?} not supported")),
        }
    }

    pub fn mul(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(a * b)),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(VarValue::Int(a * b)),
            (VarValue::Float(a), VarValue::Int(b)) => Ok(VarValue::Float(a * *b as f32)),
            (VarValue::Int(a), VarValue::Float(b)) => Ok(VarValue::Float(b * *a as f32)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a * *b)),
            (VarValue::Vec2(a), VarValue::Float(b)) => Ok(VarValue::Vec2(*a * *b)),
            _ => Err(anyhow!("{a:?} * {b:?} not supported")),
        }
    }

    pub fn div(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        if VarValue::Int(0).eq(b) {
            return Err(anyhow!("{a:?} / {b:?} division by zero"));
        }
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(a / b)),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(VarValue::Int(a / b)),
            (VarValue::Float(a), VarValue::Int(b)) => Ok(VarValue::Float(a / *b as f32)),
            (VarValue::Int(a), VarValue::Float(b)) => Ok(VarValue::Float(*a as f32 / b)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a / *b)),
            (VarValue::Vec2(a), VarValue::Float(b)) => Ok(VarValue::Vec2(*a / *b)),
            _ => Err(anyhow!("{a:?} / {b:?} not supported")),
        }
    }

    pub fn compare(a: &VarValue, b: &VarValue) -> Result<Ordering> {
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(a.total_cmp(b)),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(a.cmp(b)),
            (VarValue::Bool(a), VarValue::Bool(b)) => Ok(a.cmp(b)),
            _ => Err(anyhow!("Comparing {a:?} and {b:?} not supported")),
        }
    }

    pub fn min(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(a.min(*b))),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(VarValue::Int(*(a.min(b)))),
            (VarValue::Bool(a), VarValue::Bool(b)) => Ok(VarValue::Bool(*a && *b)),
            _ => Err(anyhow!("Comparing {a:?} and {b:?} not supported")),
        }
    }

    pub fn max(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(a.max(*b))),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(VarValue::Int(*(a.max(b)))),
            (VarValue::Bool(a), VarValue::Bool(b)) => Ok(VarValue::Bool(*a || *b)),
            _ => Err(anyhow!("Comparing {a:?} and {b:?} not supported")),
        }
    }

    pub fn abs(self) -> Result<VarValue> {
        match self {
            VarValue::Float(x) => Ok(VarValue::Float(x.abs())),
            VarValue::Int(x) => Ok(VarValue::Int(x.abs())),
            VarValue::Vec2(x) => Ok(VarValue::Vec2(x.abs())),
            _ => Err(anyhow!("Abs {self:?} not supported")),
        }
    }
}

impl Display for VarValue {
    fn fmt(&self, f: &mut __private::Formatter<'_>) -> std::fmt::Result {
        match self {
            VarValue::Float(v) => write!(f, "{v:.2}"),
            VarValue::Int(v) => write!(f, "{v}"),
            VarValue::Vec2(v) => write!(f, "{:.2}:{:.2}", v.x, v.y),
            VarValue::Bool(v) => write!(f, "{v}"),
            VarValue::String(v) => write!(f, "{v}"),
            VarValue::Faction(v) => write!(f, "{v}"),
            VarValue::Entity(v) => write!(f, "{v:?}"),
            VarValue::EntityList(v) => {
                write!(f, "[{}]", v.iter().map(|v| format!("{v:?}")).join(", "))
            }
            VarValue::Color(v) => write!(f, "{v:?}"),
            VarValue::None => write!(f, "none"),
        }
    }
}

impl std::hash::Hash for VarValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            VarValue::None => core::mem::discriminant(self).hash(state),
            VarValue::Float(v) => (*v).ord().hash(state),
            VarValue::Int(v) => (*v).hash(state),
            VarValue::Vec2(Vec2 { x, y }) => {
                (*x).ord().hash(state);
                (*y).ord().hash(state);
            }
            VarValue::Bool(v) => (*v).hash(state),
            VarValue::String(v) => (*v).hash(state),
            VarValue::Faction(v) => (*v).hash(state),
            VarValue::Entity(v) => (*v).to_bits().hash(state),
            VarValue::EntityList(v) => {
                for v in v {
                    (*v).to_bits().hash(state)
                }
            }
            VarValue::Color(v) => {
                v.r().ord().hash(state);
                v.g().ord().hash(state);
                v.b().ord().hash(state);
            }
        };
    }
}

impl VarName {
    pub fn show_editor(&mut self, id: impl std::hash::Hash, ui: &mut Ui) {
        ComboBox::from_id_source(id)
            .selected_text(self.to_string())
            .show_ui(ui, |ui| {
                for option in VarName::iter() {
                    let text = option.to_string();
                    ui.selectable_value(self, option, text);
                }
            });
    }

    pub fn show_editor_with_context(
        &mut self,
        context: &Context,
        id: impl std::hash::Hash,
        world: &World,
        ui: &mut Ui,
    ) {
        ComboBox::from_id_source(id)
            .selected_text(self.to_string())
            .show_ui(ui, |ui| {
                for option in VarName::iter() {
                    if context.get_var(option, world).is_some() {
                        let text = option
                            .to_string()
                            .add_color(white())
                            .rich_text(ui)
                            .size(10.0);
                        ui.selectable_value(self, option, text);
                    }
                }
                for option in VarName::iter() {
                    if context.get_var(option, world).is_none() {
                        let text = option.to_string().to_colored().rich_text(ui).size(10.0);
                        ui.selectable_value(self, option, text);
                    }
                }
            });
    }
}
