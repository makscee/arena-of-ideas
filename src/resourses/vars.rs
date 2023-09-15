use anyhow::anyhow;
use strum_macros::Display;

use super::*;

#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Clone, Copy, Debug, Reflect, Display)]
pub enum VarName {
    Size,
    Scale,
    Radius,
    Position,
    Rotation,
    Hp,
    Atk,
    House,
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
    Value,
    G,
    LastAttacker,
}

#[derive(Serialize, Deserialize, Clone, Debug, Reflect, PartialEq)]
pub enum VarValue {
    Float(f32),
    Int(i32),
    Vec2(Vec2),
    Bool(bool),
    String(String),
    Faction(Faction),
    Entity(Entity),
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
            _ => Err(anyhow!("String not supported by {self:?}")),
        }
    }
    pub fn get_faction(&self) -> Result<Faction> {
        match self {
            VarValue::Faction(value) => Ok(*value),
            _ => Err(anyhow!("Faction not supported by {self:?}")),
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
}
