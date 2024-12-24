use std::cmp::Ordering;

use ecolor::Color32;
use error::ExpressionError;
use glam::{vec2, Vec2};

use super::*;

#[allow(non_camel_case_types)]
#[derive(Clone, Serialize, Deserialize, Debug, AsRefStr)]
pub enum VarValue {
    i32(i32),
    f32(f32),
    u64(u64),
    bool(bool),
    String(String),
    Vec2(Vec2),
    Color32(Color32),
    Entity(u64),
}

impl VarValue {
    pub fn get_string(&self) -> Result<String, ExpressionError> {
        match self {
            VarValue::i32(v) => Ok(v.to_string()),
            VarValue::f32(v) => Ok(v.to_string()),
            VarValue::u64(v) => Ok(v.to_string()),
            VarValue::bool(v) => Ok(v.to_string()),
            VarValue::String(v) => Ok(v.to_string()),
            VarValue::Vec2(v) => Ok(v.to_string()),
            VarValue::Color32(v) => Ok(v.to_hex()),
            VarValue::Entity(v) => Ok(v.to_string()),
        }
    }
    pub fn get_i32(&self) -> Result<i32, ExpressionError> {
        match self {
            VarValue::i32(v) => Ok(*v),
            VarValue::f32(v) => Ok(*v as i32),
            VarValue::bool(v) => Ok(*v as i32),
            _ => Err(ExpressionError::not_supported_single(
                "Cast to i32",
                self.clone(),
            )),
        }
    }
    pub fn get_f32(&self) -> Result<f32, ExpressionError> {
        match self {
            VarValue::f32(v) => Ok(*v),
            VarValue::i32(v) => Ok(*v as f32),
            VarValue::u64(v) => Ok(*v as f32),
            VarValue::bool(v) => Ok(*v as i32 as f32),
            _ => Err(ExpressionError::not_supported_single(
                "Cast to f32",
                self.clone(),
            )),
        }
    }
    pub fn get_bool(&self) -> Result<bool, ExpressionError> {
        match self {
            VarValue::bool(v) => Ok(*v),
            VarValue::i32(v) => Ok(*v > 0),
            VarValue::f32(v) => Ok(*v > 0.0),
            VarValue::String(v) => Ok(!v.is_empty()),
            _ => Err(ExpressionError::not_supported_single(
                "Cast to bool",
                self.clone(),
            )),
        }
    }
    pub fn get_u64(&self) -> Result<u64, ExpressionError> {
        match self {
            VarValue::u64(v) => Ok(*v),
            _ => Err(ExpressionError::not_supported_single(
                "Cast to u64",
                self.clone(),
            )),
        }
    }
    pub fn get_vec2(&self) -> Result<Vec2, ExpressionError> {
        match self {
            VarValue::Vec2(v) => Ok(*v),
            VarValue::f32(v) => Ok(vec2(*v, *v)),
            _ => Err(ExpressionError::not_supported_single(
                "Cast to Vec2",
                self.clone(),
            )),
        }
    }
    pub fn get_color(&self) -> Result<Color32, ExpressionError> {
        match self {
            VarValue::Color32(v) => Ok(*v),
            VarValue::String(v) => Ok(Color32::from_hex(v)
                .unwrap_or(Color32::from_rgb(255, 0, 255))
                .into()),
            _ => Err(ExpressionError::not_supported_single(
                "Cast to Color32",
                self.clone(),
            )),
        }
    }
    pub fn compare(a: &VarValue, b: &VarValue) -> Result<Ordering, ExpressionError> {
        match (a, b) {
            (VarValue::f32(a), VarValue::f32(b)) => Ok(a.total_cmp(b)),
            (VarValue::i32(a), VarValue::i32(b)) => Ok(a.cmp(b)),
            (VarValue::u64(a), VarValue::u64(b)) => Ok(a.cmp(b)),
            (VarValue::bool(a), VarValue::bool(b)) => Ok(a.cmp(b)),
            (VarValue::String(a), VarValue::String(b)) => Ok(a.cmp(b)),
            _ => Err(ExpressionError::not_supported_multiple(
                "Compare",
                vec![a.clone(), b.clone()],
            )),
        }
    }
    pub fn add(&self, b: &VarValue) -> Result<Self, ExpressionError> {
        let a = self;
        match (a, b) {
            (VarValue::String(a), ..) => {
                Ok(VarValue::String(a.to_owned() + b.get_string()?.as_str()))
            }
            (.., VarValue::String(b)) => Ok(VarValue::String(a.get_string()? + b.as_str())),
            (VarValue::f32(a), ..) => Ok(VarValue::f32(a + b.get_f32()?)),
            (.., VarValue::f32(b)) => Ok(VarValue::f32(a.get_f32()? + *b)),
            (VarValue::i32(a), ..) => Ok(VarValue::i32(a + b.get_i32()?)),
            (.., VarValue::i32(b)) => Ok(VarValue::i32(a.get_i32()? + *b)),
            (VarValue::bool(a), VarValue::bool(b)) => Ok(VarValue::bool(*a || *b)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a + *b)),
            _ => Err(ExpressionError::not_supported_multiple(
                "Add",
                vec![a.clone(), b.clone()],
            )),
        }
    }
    pub fn sub(&self, b: &VarValue) -> Result<Self, ExpressionError> {
        let a = self;
        match (a, b) {
            (VarValue::f32(a), VarValue::f32(b)) => Ok(VarValue::f32(a - b)),
            (VarValue::i32(a), VarValue::i32(b)) => Ok(VarValue::i32(a - b)),
            (VarValue::f32(a), VarValue::i32(b)) => Ok(VarValue::f32(a - *b as f32)),
            (VarValue::i32(a), VarValue::f32(b)) => Ok(VarValue::f32(*a as f32 - b)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a - *b)),
            _ => Err(ExpressionError::not_supported_multiple(
                "Sub",
                vec![a.clone(), b.clone()],
            )),
        }
    }
    pub fn mul(&self, b: &VarValue) -> Result<Self, ExpressionError> {
        let a = self;
        match (a, b) {
            (VarValue::f32(a), VarValue::f32(b)) => Ok(VarValue::f32(a * b)),
            (VarValue::i32(a), VarValue::i32(b)) => Ok(VarValue::i32(a * b)),
            (VarValue::f32(a), VarValue::i32(b)) => Ok(VarValue::f32(a * *b as f32)),
            (VarValue::i32(a), VarValue::f32(b)) => Ok(VarValue::f32(b * *a as f32)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a * *b)),
            (VarValue::Vec2(a), VarValue::f32(b)) => Ok(VarValue::Vec2(*a * *b)),
            (VarValue::f32(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a * *b)),
            _ => Err(ExpressionError::not_supported_multiple(
                "Mul",
                vec![a.clone(), b.clone()],
            )),
        }
    }
    pub fn div(&self, b: &VarValue) -> Result<Self, ExpressionError> {
        let a = self;
        if VarValue::i32(0).eq(b) {
            return Err(ExpressionError::not_supported_multiple(
                "Div by zero",
                vec![a.clone(), b.clone()],
            ));
        }
        match (a, b) {
            (VarValue::f32(a), VarValue::f32(b)) => Ok(VarValue::f32(a / b)),
            (VarValue::i32(a), VarValue::i32(b)) => Ok(VarValue::i32(a / b)),
            (VarValue::f32(a), VarValue::i32(b)) => Ok(VarValue::f32(a / *b as f32)),
            (VarValue::i32(a), VarValue::f32(b)) => Ok(VarValue::f32(*a as f32 / b)),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a / *b)),
            (VarValue::Vec2(a), VarValue::f32(b)) => Ok(VarValue::Vec2(*a / *b)),
            _ => Err(ExpressionError::not_supported_multiple(
                "Div",
                vec![a.clone(), b.clone()],
            )),
        }
    }
    pub fn min(&self, b: &VarValue) -> Result<Self, ExpressionError> {
        let a = self;
        match (a, b) {
            (VarValue::f32(a), VarValue::f32(b)) => Ok(VarValue::f32(a.min(*b))),
            (VarValue::i32(a), VarValue::i32(b)) => Ok(VarValue::i32(*(a.min(b)))),
            (VarValue::bool(a), VarValue::bool(b)) => Ok(VarValue::bool(*a && *b)),
            _ => Err(ExpressionError::not_supported_multiple(
                "Min",
                vec![a.clone(), b.clone()],
            )),
        }
    }
    pub fn max(&self, b: &VarValue) -> Result<Self, ExpressionError> {
        let a = self;
        match (a, b) {
            (VarValue::f32(a), VarValue::f32(b)) => Ok(VarValue::f32(a.max(*b))),
            (VarValue::i32(a), VarValue::i32(b)) => Ok(VarValue::i32(*(a.max(b)))),
            (VarValue::bool(a), VarValue::bool(b)) => Ok(VarValue::bool(*a || *b)),
            _ => Err(ExpressionError::not_supported_multiple(
                "Max",
                vec![a.clone(), b.clone()],
            )),
        }
    }
    pub fn abs(self) -> Result<Self, ExpressionError> {
        match self {
            VarValue::f32(x) => Ok(VarValue::f32(x.abs())),
            VarValue::i32(x) => Ok(VarValue::i32(x.abs())),
            VarValue::Vec2(x) => Ok(VarValue::Vec2(x.abs())),
            _ => Err(ExpressionError::not_supported_single("Abs", self.clone())),
        }
    }
}

impl Default for VarValue {
    fn default() -> Self {
        Self::i32(0)
    }
}

impl std::hash::Hash for VarValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            VarValue::i32(v) => v.hash(state),
            VarValue::f32(v) => v.to_bits().hash(state),
            VarValue::u64(v) => v.hash(state),
            VarValue::bool(v) => v.hash(state),
            VarValue::String(v) => v.hash(state),
            VarValue::Vec2(v) => {
                v.x.to_bits().hash(state);
                v.y.to_bits().hash(state);
            }
            VarValue::Color32(v) => v.hash(state),
            VarValue::Entity(v) => v.hash(state),
        };
    }
}
impl PartialEq for VarValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (VarValue::i32(a), VarValue::i32(b)) => a == b,
            (VarValue::f32(a), VarValue::f32(b)) => a == b,
            (VarValue::u64(a), VarValue::u64(b)) => a == b,
            (VarValue::bool(a), VarValue::bool(b)) => a == b,
            (VarValue::String(a), VarValue::String(b)) => a == b,
            (VarValue::Vec2(a), VarValue::Vec2(b)) => a == b,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl std::fmt::Display for VarValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VarValue::i32(v) => write!(f, "{v}"),
            VarValue::u64(v) => write!(f, "{v}"),
            VarValue::f32(v) => write!(f, "{v:.2}"),
            VarValue::bool(v) => write!(f, "{v}"),
            VarValue::String(v) => write!(f, "{v}"),
            VarValue::Vec2(v) => write!(f, "{:.2}, {:.2}", v.x, v.y),
            VarValue::Color32(v) => write!(f, "{}", v.to_hex()),
            VarValue::Entity(v) => write!(f, "{v}"),
        }
    }
}

impl From<i32> for VarValue {
    fn from(value: i32) -> Self {
        VarValue::i32(value)
    }
}
impl Into<i32> for VarValue {
    fn into(self) -> i32 {
        self.get_i32().unwrap()
    }
}
impl From<usize> for VarValue {
    fn from(value: usize) -> Self {
        VarValue::i32(value as i32)
    }
}
impl Into<usize> for VarValue {
    fn into(self) -> usize {
        self.get_i32().unwrap() as usize
    }
}
impl From<f32> for VarValue {
    fn from(value: f32) -> Self {
        VarValue::f32(value)
    }
}
impl Into<f32> for VarValue {
    fn into(self) -> f32 {
        self.get_f32().unwrap()
    }
}
impl From<u64> for VarValue {
    fn from(value: u64) -> Self {
        VarValue::u64(value)
    }
}
impl Into<u64> for VarValue {
    fn into(self) -> u64 {
        self.get_u64().unwrap()
    }
}
impl From<bool> for VarValue {
    fn from(value: bool) -> Self {
        VarValue::bool(value)
    }
}
impl Into<bool> for VarValue {
    fn into(self) -> bool {
        self.get_bool().unwrap()
    }
}
impl From<String> for VarValue {
    fn from(value: String) -> Self {
        VarValue::String(value)
    }
}
impl Into<String> for VarValue {
    fn into(self) -> String {
        self.get_string().unwrap()
    }
}
impl From<Color32> for VarValue {
    fn from(value: Color32) -> Self {
        VarValue::Color32(value)
    }
}
impl Into<Color32> for VarValue {
    fn into(self) -> Color32 {
        self.get_color().unwrap()
    }
}
impl From<Vec2> for VarValue {
    fn from(value: Vec2) -> Self {
        VarValue::Vec2(value)
    }
}
impl Into<Vec2> for VarValue {
    fn into(self) -> Vec2 {
        self.get_vec2().unwrap()
    }
}
