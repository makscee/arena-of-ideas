use super::*;
use ::tween::*;
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
    pub fn f(
        &self,
        a: &VarValue,
        b: &VarValue,
        t: f32,
        over: f32,
    ) -> Result<VarValue, ExpressionError> {
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
            (VarValue::f32(a), VarValue::f32(b)) => Ok(VarValue::f32(*a + (*b - *a) * t)),
            (VarValue::i32(a), VarValue::i32(b)) => {
                Ok(VarValue::i32(*a + ((*b - *a) as f32 * t) as i32))
            }
            (VarValue::String(a), VarValue::String(b)) => Ok(VarValue::String(match t > 0.5 {
                true => a.into(),
                false => b.into(),
            })),
            (VarValue::Vec2(a), VarValue::Vec2(b)) => Ok(VarValue::Vec2(*a + (*b - *a) * t)),
            (VarValue::Color32(a), VarValue::Color32(b)) => {
                Ok(a.to_color().mix(&b.to_color(), t).c32().into())
            }
            (VarValue::bool(a), VarValue::bool(b)) => Ok(VarValue::bool(match t > 0.5 {
                true => *a,
                false => *b,
            })),
            _ => Err(ExpressionError::not_supported_multiple(
                "Tween",
                vec![a.clone(), b.clone()],
            )),
        }
    }
}
