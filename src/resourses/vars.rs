use std::cmp::Ordering;

use anyhow::anyhow;
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
    Value,
    Size,
    Scale,
    Radius,
    Position,
    Rotation,
    Hp,
    Atk,
    House,
    HouseColor,
    Dmg,
    Name,
    Description,
    Text,
    Spawn,
    Slot,
    Faction,
    Visible,
    Direction,
    Charges,
    G,
    LastAttacker,
    Color,
    Thickness,
    Curvature,
    Delta,
    T,
    Alpha,
    Index,
    IncomingDamage,
    OutgoingDamage,
}

#[derive(Serialize, Deserialize, Clone, Debug, Reflect, PartialEq, Display)]
pub enum VarValue {
    Float(f32),
    Int(i32),
    Vec2(Vec2),
    Bool(bool),
    String(String),
    Faction(Faction),
    Entity(Entity),
    Color(Color),
}

impl VarValue {
    pub fn get_float(&self) -> Result<f32> {
        match self {
            VarValue::Float(value) => Ok(*value),
            VarValue::Int(value) => Ok(*value as f32),
            _ => Err(anyhow!("Float not supported by {self:?}")),
        }
    }
    pub fn get_int(&self) -> Result<i32> {
        match self {
            VarValue::Int(value) => Ok(*value),
            _ => Err(anyhow!("Int not supported by {self:?}")),
        }
    }
    pub fn get_vec2(&self) -> Result<Vec2> {
        match self {
            VarValue::Vec2(value) => Ok(*value),
            _ => Err(anyhow!("Vec2 not supported by {self:?}")),
        }
    }
    pub fn get_bool(&self) -> Result<bool> {
        match self {
            VarValue::Bool(value) => Ok(*value),
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

    pub fn sum(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(a + b)),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(VarValue::Int(a + b)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a + *b)),
            (VarValue::String(a), VarValue::String(b)) => Ok(VarValue::String(a.to_owned() + b)),
            _ => Err(anyhow!("{a:?} + {b:?} not supported")),
        }
    }

    pub fn sub(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(a - b)),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(VarValue::Int(a - b)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a - *b)),
            _ => Err(anyhow!("{a:?} - {b:?} not supported")),
        }
    }

    pub fn mul(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(a * b)),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(VarValue::Int(a * b)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a * *b)),
            (VarValue::Vec2(a), VarValue::Float(b)) => Ok(VarValue::Vec2(*a * *b)),
            _ => Err(anyhow!("{a:?} * {b:?} not supported")),
        }
    }

    pub fn div(a: &VarValue, b: &VarValue) -> Result<VarValue> {
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(a / b)),
            (VarValue::Int(a), VarValue::Int(b)) => Ok(VarValue::Int(a / b)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a / *b)),
            (VarValue::Vec2(a), VarValue::Float(b)) => Ok(VarValue::Vec2(*a / *b)),
            _ => Err(anyhow!("{a:?} * {b:?} not supported")),
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

impl VarName {
    pub fn show_editor(&mut self, ui: &mut Ui) {
        ComboBox::from_id_source(*self)
            .selected_text(self.to_string())
            .show_ui(ui, |ui| {
                for option in VarName::iter() {
                    let text = option.to_string();
                    ui.selectable_value(self, option, text);
                }
            });
    }
}
