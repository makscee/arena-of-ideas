use super::*;
use ::tween;
use bevy::color::Mix;

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
    pub fn f(&self, a: &VarValue, b: &VarValue, t: f32, over: f32) -> Result<VarValue> {
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
        let t = match self {
            Tween::Linear => tween::Tweener::linear(0.0, 1.0, 1.0).move_to(t),
            Tween::QuartOut => tween::Tweener::quart_out(0.0, 1.0, 1.0).move_to(t),
            Tween::QuartIn => tween::Tweener::quart_in(0.0, 1.0, 1.0).move_to(t),
            Tween::QuartInOut => tween::Tweener::quart_in_out(0.0, 1.0, 1.0).move_to(t),
            Tween::QuadOut => tween::Tweener::quad_out(0.0, 1.0, 1.0).move_to(t),
            Tween::QuadIn => tween::Tweener::quad_in(0.0, 1.0, 1.0).move_to(t),
            Tween::QuadInOut => tween::Tweener::quad_in_out(0.0, 1.0, 1.0).move_to(t),
            Tween::CubicIn => tween::Tweener::cubic_in(0.0, 1.0, 1.0).move_to(t),
            Tween::CubicOut => tween::Tweener::cubic_out(0.0, 1.0, 1.0).move_to(t),
            Tween::BackIn => tween::Tweener::back_in(0.0, 1.0, 1.0).move_to(t),
        };
        match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => Ok(VarValue::Float(*a + (*b - *a) * t)),
            (VarValue::Int(a), VarValue::Int(b)) => {
                Ok(VarValue::Int(*a + ((*b - *a) as f32 * t) as i32))
            }
            (VarValue::String(a), VarValue::String(b)) => Ok(VarValue::String(match t > 0.5 {
                true => a.into(),
                false => b.into(),
            })),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a + (*b - *a) * t)),
            (VarValue::Color(a), VarValue::Color(b)) => Ok(a.mix(b, t).into()),
            (VarValue::Bool(a), VarValue::Bool(b)) => Ok(VarValue::Bool(match t > 0.5 {
                true => *a,
                false => *b,
            })),
            _ => Err(anyhow!("Tweening not supported for {a:?} and {b:?}")),
        }
    }
}

impl ToCstr for Tween {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr()
    }
}
