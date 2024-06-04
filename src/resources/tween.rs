use super::*;
use ::tween;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq)]
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
        let t = t / over;
        if t.is_nan() || t <= 0.0 {
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
        let v = match (a, b) {
            (VarValue::Float(a), VarValue::Float(b)) => VarValue::Float(*a + (*b - *a) * t),
            (VarValue::Int(a), VarValue::Int(b)) => {
                VarValue::Int(*a + ((*b - *a) as f32 * t) as i32)
            }
            (VarValue::String(a), VarValue::String(b)) => VarValue::String(match t > 0.5 {
                true => a.into(),
                false => b.into(),
            }),
            // (VarValue::Vec2(a), VarValue::Vec2(b)) => VarValue::Vec2(*a + (*b - *a) * t),
            // (VarValue::Color(a), VarValue::Color(b)) => {
            //     let mut sub = *b;
            //     sub.set_r(b.r() - a.r());
            //     sub.set_g(b.g() - a.g());
            //     sub.set_b(b.b() - a.b());
            //     sub.set_a(b.a() - a.a());
            //     VarValue::Color(*a + sub * t)
            // }
            // (VarValue::Bool(a), VarValue::Bool(b)) => VarValue::Bool(match t > 0.5 {
            //     true => *a,
            //     false => *b,
            // }),
            _ => panic!("Tweening not supported for {a:?} and {b:?}"),
        };
        Ok(v)
    }
}
