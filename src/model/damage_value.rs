use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(try_from = "String")]
pub struct DamageValue {
    pub absolute: Health,
    pub relative: R32,
}

impl DamageValue {
    pub const ZERO: Self = Self {
        absolute: Health::ZERO,
        relative: R32::ZERO,
    };

    pub fn absolute(value: f32) -> Self {
        Self {
            absolute: Health::new(value),
            relative: R32::ZERO,
        }
    }
    pub fn relative(value: f32) -> Self {
        Self {
            absolute: Health::new(0.0),
            relative: R32::new(value),
        }
    }
}

impl Mul<R32> for DamageValue {
    type Output = Self;
    fn mul(self, rhs: R32) -> Self {
        Self {
            absolute: self.absolute * rhs,
            relative: self.relative * rhs,
        }
    }
}

impl Add<Health> for DamageValue {
    type Output = Self;
    fn add(self, rhs: R32) -> Self {
        Self {
            absolute: self.absolute + rhs,
            relative: self.relative,
        }
    }
}

impl Default for DamageValue {
    fn default() -> Self {
        Self::ZERO
    }
}

impl TryFrom<String> for DamageValue {
    type Error = <f32 as std::str::FromStr>::Err;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.ends_with('%') {
            let percent = R32::new(value[..value.len() - 1].parse()?);
            Ok(Self {
                absolute: Health::ZERO,
                relative: percent,
            })
        } else {
            let value = Health::new(value.parse()?);
            Ok(Self {
                absolute: value,
                relative: R32::ZERO,
            })
        }
    }
}
