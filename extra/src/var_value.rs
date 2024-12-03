use std::cmp::Ordering;

use super::*;

#[allow(non_camel_case_types)]
#[derive(Clone, Serialize, Deserialize, Debug, AsRefStr, Display)]
pub enum VarValue {
    i32(i32),
    f32(f32),
    u64(u64),
    bool(bool),
    String(String),
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
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
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
