use std::cmp::Ordering;

use bevy::color::Srgba;

use super::*;

#[allow(non_camel_case_types)]
#[derive(Clone, Serialize, Deserialize, Debug, AsRefStr, Reflect)]
pub enum VarValue {
    i32(i32),
    f32(f32),
    u64(u64),
    bool(bool),
    String(String),
    Vec2(Vec2),
    Color(Color),
}

#[derive(Error, Debug)]
pub enum VarValueError {
    #[error("Cast to {t} not supported by {value}")]
    CastNotSupported { value: VarValue, t: &'static str },
    #[error("Comparing {a} and {b} not supported")]
    CompareNotSupported { a: VarValue, b: VarValue },
}

impl VarValue {
    pub fn get_string(&self) -> Result<String, VarValueError> {
        match self {
            VarValue::i32(v) => Ok(v.to_string()),
            VarValue::f32(v) => Ok(v.to_string()),
            VarValue::u64(v) => Ok(v.to_string()),
            VarValue::bool(v) => Ok(v.to_string()),
            VarValue::String(v) => Ok(v.to_string()),
            VarValue::Vec2(v) => Ok(v.to_string()),
            VarValue::Color(color) => Ok(color.to_srgba().to_hex()),
        }
    }
    pub fn get_i32(&self) -> Result<i32, VarValueError> {
        match self {
            VarValue::i32(v) => Ok(*v),
            VarValue::f32(v) => Ok(*v as i32),
            VarValue::bool(v) => Ok(*v as i32),
            _ => Err(VarValueError::CastNotSupported {
                value: self.clone(),
                t: "i32",
            }),
        }
    }
    pub fn get_u64(&self) -> Result<u64, VarValueError> {
        match self {
            VarValue::u64(v) => Ok(*v),
            _ => Err(VarValueError::CastNotSupported {
                value: self.clone(),
                t: "u64",
            }),
        }
    }
    pub fn get_vec2(&self) -> Result<Vec2, VarValueError> {
        match self {
            VarValue::Vec2(v) => Ok(*v),
            _ => Err(VarValueError::CastNotSupported {
                value: self.clone(),
                t: "Vec2",
            }),
        }
    }
    pub fn get_color(&self) -> Result<Color, VarValueError> {
        match self {
            VarValue::Color(v) => Ok(*v),
            VarValue::String(v) => Ok(Srgba::hex(v).unwrap_or(BEVY_MISSING_COLOR.into()).into()),
            _ => Err(VarValueError::CastNotSupported {
                value: self.clone(),
                t: "Color",
            }),
        }
    }
    pub fn compare(a: &VarValue, b: &VarValue) -> Result<Ordering, VarValueError> {
        match (a, b) {
            (VarValue::f32(a), VarValue::f32(b)) => Ok(a.total_cmp(b)),
            (VarValue::i32(a), VarValue::i32(b)) => Ok(a.cmp(b)),
            (VarValue::u64(a), VarValue::u64(b)) => Ok(a.cmp(b)),
            (VarValue::bool(a), VarValue::bool(b)) => Ok(a.cmp(b)),
            (VarValue::String(a), VarValue::String(b)) => Ok(a.cmp(b)),
            _ => Err(VarValueError::CompareNotSupported {
                a: a.clone(),
                b: b.clone(),
            }),
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
            VarValue::Color(color) => color.reflect_hash().unwrap().hash(state),
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
            VarValue::i32(v) => write!(f, "{}", v),
            VarValue::u64(v) => write!(f, "{}", v),
            VarValue::f32(v) => write!(f, "{:.2}", v),
            VarValue::bool(v) => write!(f, "{}", v),
            VarValue::String(v) => write!(f, "{}", v),
            VarValue::Vec2(v) => write!(f, "{:.2}, {:.2}", v.x, v.y),
            VarValue::Color(color) => write!(f, "{}", color.to_srgba().to_hex()),
        }
    }
}

impl From<i32> for VarValue {
    fn from(value: i32) -> Self {
        VarValue::i32(value)
    }
}
impl From<f32> for VarValue {
    fn from(value: f32) -> Self {
        VarValue::f32(value)
    }
}
impl From<u64> for VarValue {
    fn from(value: u64) -> Self {
        VarValue::u64(value)
    }
}
impl From<String> for VarValue {
    fn from(value: String) -> Self {
        VarValue::String(value)
    }
}
impl From<Color> for VarValue {
    fn from(value: Color) -> Self {
        VarValue::Color(value)
    }
}
impl From<Vec2> for VarValue {
    fn from(value: Vec2) -> Self {
        VarValue::Vec2(value)
    }
}
