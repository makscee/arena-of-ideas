use crate::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, AsRefStr, EnumIter)]
pub enum Tween {
    #[default]
    Linear,
    QuartOut,
    QuartIn,
    QuartInOut,
    QuadOut,
    QuadIn,
    QuadInOut,
    CubicIn,
    CubicOut,
    BackIn,
}

impl Tween {
    pub fn f(&self, a: &VarValue, b: &VarValue, t: f32, over: f32) -> Result<VarValue, NodeError> {
        if over == 0.0 {
            return Ok(b.clone());
        }
        let t = t / over;
        if t <= 0.0 {
            return Ok(a.clone());
        }
        if t >= 1.0 {
            return Ok(b.clone());
        }

        // Calculate eased t value
        let eased_t = match self {
            Tween::Linear => t,
            Tween::QuartOut => 1.0 - (1.0 - t).powi(4),
            Tween::QuartIn => t.powi(4),
            Tween::QuartInOut => {
                if t < 0.5 {
                    8.0 * t.powi(4)
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(4) / 2.0
                }
            }
            Tween::QuadOut => 1.0 - (1.0 - t).powi(2),
            Tween::QuadIn => t.powi(2),
            Tween::QuadInOut => {
                if t < 0.5 {
                    2.0 * t.powi(2)
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
            Tween::CubicIn => t.powi(3),
            Tween::CubicOut => 1.0 - (1.0 - t).powi(3),
            Tween::BackIn => {
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;
                C3 * t * t * t - C1 * t * t
            }
        };

        match (a, b) {
            (VarValue::f32(a), VarValue::f32(b)) => Ok(VarValue::f32(*a + (*b - *a) * eased_t)),
            (VarValue::i32(a), VarValue::i32(b)) => {
                Ok(VarValue::i32(*a + ((*b - *a) as f32 * eased_t) as i32))
            }
            (VarValue::String(a), VarValue::String(b)) => {
                Ok(VarValue::String(match eased_t > 0.5 {
                    true => a.clone(),
                    false => b.clone(),
                }))
            }
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a + (*b - *a) * eased_t)),
            (VarValue::Color32(a), VarValue::Color32(b)) => {
                // Simple linear interpolation for colors without bevy::color::Mix
                let a_r = a.r() as f32;
                let a_g = a.g() as f32;
                let a_b = a.b() as f32;
                let a_a = a.a() as f32;

                let b_r = b.r() as f32;
                let b_g = b.g() as f32;
                let b_b = b.b() as f32;
                let b_a = b.a() as f32;

                let r = (a_r + (b_r - a_r) * eased_t) as u8;
                let g = (a_g + (b_g - a_g) * eased_t) as u8;
                let b_val = (a_b + (b_b - a_b) * eased_t) as u8;
                let a_val = (a_a + (b_a - a_a) * eased_t) as u8;

                Ok(VarValue::Color32(Color32::from_rgba_premultiplied(
                    r, g, b_val, a_val,
                )))
            }
            (VarValue::bool(a), VarValue::bool(b)) => Ok(VarValue::bool(match eased_t > 0.5 {
                true => *a,
                false => *b,
            })),
            (VarValue::u64(a), VarValue::u64(b)) => {
                Ok(VarValue::u64(*a + ((*b - *a) as f32 * eased_t) as u64))
            }
            _ => Err(NodeError::not_supported_multiple(
                "Tween",
                vec![a.clone(), b.clone()],
            )),
        }
    }
}
