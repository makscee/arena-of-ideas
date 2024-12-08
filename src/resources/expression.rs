use bevy::color::Srgba;

use super::*;

pub trait ExpressionImpl {
    fn get_value(&self, e: Entity, context: &Context) -> Result<VarValue, ExpressionError>;
    fn get_f32(&self, e: Entity, context: &Context) -> Result<f32, ExpressionError>;
    fn get_i32(&self, e: Entity, context: &Context) -> Result<i32, ExpressionError>;
    fn get_vec2(&self, e: Entity, context: &Context) -> Result<Vec2, ExpressionError>;
    fn get_bool(&self, e: Entity, context: &Context) -> Result<bool, ExpressionError>;
    fn get_color(&self, e: Entity, context: &Context) -> Result<Color, ExpressionError>;
    fn get_string(&self, e: Entity, context: &Context) -> Result<String, ExpressionError>;
}

impl ExpressionImpl for Expression {
    fn get_value(&self, e: Entity, context: &Context) -> Result<VarValue, ExpressionError> {
        match self {
            Expression::One => Ok(1.into()),
            Expression::Zero => Ok(0.into()),
            Expression::Var(var) => context.get_var(*var).to_e(),
            Expression::V(v) => Ok(v.clone()),
            Expression::F(v) => Ok((*v).into()),
            Expression::I(v) => Ok((*v).into()),
            Expression::B(v) => Ok((*v).into()),
            Expression::V2(x, y) => Ok(vec2(*x, *y).into()),
            Expression::S(s) => Ok(s.clone().into()),
            Expression::C(s) => Srgba::hex(s)
                .map_err(|e| ExpressionError::HexColorError(e))
                .map(|v| VarValue::Color(v.into())),
            Expression::GT => Ok(gt().play_head().into()),
            Expression::Sin(x) => Ok(x.get_f32(e, context)?.sin().into()),
            Expression::Cos(x) => Ok(x.get_f32(e, context)?.cos().into()),
            Expression::Even(x) => Ok((x.get_i32(e, context)? % 2 == 0).into()),
            Expression::Abs(x) => x.get_value(e, context)?.abs(),
            Expression::Floor(x) => Ok(x.get_f32(e, context)?.floor().into()),
            Expression::Ceil(x) => Ok(x.get_f32(e, context)?.ceil().into()),
            Expression::Fract(x) => Ok(x.get_f32(e, context)?.fract().into()),
            Expression::Sqr(x) => Ok({
                let x = x.get_f32(e, context)?;
                (x * x).into()
            }),
            Expression::Macro(s, v) => {
                let s = s.get_string(e, context)?;
                let v = v.get_string(e, context)?;
                Ok(s.replace("%s", &v).into())
            }
            Expression::Sum(a, b) => a.get_value(e, context)?.add(&b.get_value(e, context)?),
            Expression::Sub(a, b) => a.get_value(e, context)?.sub(&b.get_value(e, context)?),
            Expression::Mul(a, b) => a.get_value(e, context)?.mul(&b.get_value(e, context)?),
            Expression::Div(a, b) => a.get_value(e, context)?.div(&b.get_value(e, context)?),
            Expression::Max(a, b) => a.get_value(e, context)?.max(&b.get_value(e, context)?),
            Expression::Min(a, b) => a.get_value(e, context)?.min(&b.get_value(e, context)?),
            Expression::Mod(a, b) => Ok((a.get_i32(e, context)? % b.get_i32(e, context)?).into()),
            Expression::And(a, b) => {
                Ok((a.get_bool(e, context)? && b.get_bool(e, context)?).into())
            }
            Expression::Or(a, b) => Ok((a.get_bool(e, context)? || b.get_bool(e, context)?).into()),
            Expression::Equals(a, b) => {
                Ok((a.get_value(e, context)? == b.get_value(e, context)?).into())
            }
            Expression::GreaterThen(a, b) => Ok(VarValue::bool(matches!(
                VarValue::compare(&a.get_value(e, context)?, &b.get_value(e, context)?)?,
                std::cmp::Ordering::Greater
            ))),
            Expression::LessThen(a, b) => Ok(VarValue::bool(matches!(
                VarValue::compare(&a.get_value(e, context)?, &b.get_value(e, context)?)?,
                std::cmp::Ordering::Less
            ))),
            Expression::If(i, t, el) => {
                if i.get_bool(e, context)? {
                    t.get_value(e, context)
                } else {
                    el.get_value(e, context)
                }
            }
        }
    }
    fn get_f32(&self, e: Entity, context: &Context) -> Result<f32, ExpressionError> {
        self.get_value(e, context)?.get_f32()
    }
    fn get_i32(&self, e: Entity, context: &Context) -> Result<i32, ExpressionError> {
        self.get_value(e, context)?.get_i32()
    }
    fn get_vec2(&self, e: Entity, context: &Context) -> Result<Vec2, ExpressionError> {
        self.get_value(e, context)?.get_vec2()
    }
    fn get_bool(&self, e: Entity, context: &Context) -> Result<bool, ExpressionError> {
        self.get_value(e, context)?.get_bool()
    }
    fn get_color(&self, e: Entity, context: &Context) -> Result<Color, ExpressionError> {
        self.get_value(e, context)?.get_color()
    }
    fn get_string(&self, e: Entity, context: &Context) -> Result<String, ExpressionError> {
        self.get_value(e, context)?.get_string()
    }
}
