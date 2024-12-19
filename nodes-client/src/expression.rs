use super::*;

pub trait ExpressionImpl {
    fn get_value(&self, context: &Context) -> Result<VarValue, ExpressionError>;
    fn get_f32(&self, context: &Context) -> Result<f32, ExpressionError>;
    fn get_i32(&self, context: &Context) -> Result<i32, ExpressionError>;
    fn get_vec2(&self, context: &Context) -> Result<Vec2, ExpressionError>;
    fn get_bool(&self, context: &Context) -> Result<bool, ExpressionError>;
    fn get_color(&self, context: &Context) -> Result<Color32, ExpressionError>;
    fn get_string(&self, context: &Context) -> Result<String, ExpressionError>;
}

impl ExpressionImpl for Expression {
    fn get_value(&self, context: &Context) -> Result<VarValue, ExpressionError> {
        match self {
            Expression::One => Ok(1.into()),
            Expression::Zero => Ok(0.into()),
            Expression::Var(var) => {
                let v = context.get_var(*var).to_e();
                if v.is_err() && *var == VarName::index {
                    Ok(1.into())
                } else {
                    v
                }
            }
            Expression::V(v) => Ok(v.clone()),
            Expression::F(v) => Ok((*v).into()),
            Expression::I(v) => Ok((*v).into()),
            Expression::B(v) => Ok((*v).into()),
            Expression::V2(x, y) => Ok(vec2(*x, *y).into()),
            Expression::S(s) => Ok(s.clone().into()),
            Expression::C(s) => Color32::from_hex(s)
                .map_err(|e| ExpressionError::OperationNotSupported {
                    values: default(),
                    op: "Hex conversion err",
                })
                .map(|v| v.into()),
            Expression::GT => Ok(gt().play_head().into()),
            Expression::Sin(x) => Ok(x.get_f32(context)?.sin().into()),
            Expression::Cos(x) => Ok(x.get_f32(context)?.cos().into()),
            Expression::Even(x) => Ok((x.get_i32(context)? % 2 == 0).into()),
            Expression::Abs(x) => x.get_value(context)?.abs(),
            Expression::Floor(x) => Ok(x.get_f32(context)?.floor().into()),
            Expression::Ceil(x) => Ok(x.get_f32(context)?.ceil().into()),
            Expression::Fract(x) => Ok(x.get_f32(context)?.fract().into()),
            Expression::Sqr(x) => Ok({
                let x = x.get_f32(context)?;
                (x * x).into()
            }),
            Expression::Macro(s, v) => {
                let s = s.get_string(context)?;
                let v = v.get_string(context)?;
                Ok(s.replace("%s", &v).into())
            }
            Expression::Sum(a, b) => a.get_value(context)?.add(&b.get_value(context)?),
            Expression::Sub(a, b) => a.get_value(context)?.sub(&b.get_value(context)?),
            Expression::Mul(a, b) => a.get_value(context)?.mul(&b.get_value(context)?),
            Expression::Div(a, b) => a.get_value(context)?.div(&b.get_value(context)?),
            Expression::Max(a, b) => a.get_value(context)?.max(&b.get_value(context)?),
            Expression::Min(a, b) => a.get_value(context)?.min(&b.get_value(context)?),
            Expression::Mod(a, b) => Ok((a.get_i32(context)? % b.get_i32(context)?).into()),
            Expression::And(a, b) => Ok((a.get_bool(context)? && b.get_bool(context)?).into()),
            Expression::Or(a, b) => Ok((a.get_bool(context)? || b.get_bool(context)?).into()),
            Expression::Equals(a, b) => Ok((a.get_value(context)? == b.get_value(context)?).into()),
            Expression::GreaterThen(a, b) => Ok(VarValue::bool(matches!(
                VarValue::compare(&a.get_value(context)?, &b.get_value(context)?)?,
                std::cmp::Ordering::Greater
            ))),
            Expression::LessThen(a, b) => Ok(VarValue::bool(matches!(
                VarValue::compare(&a.get_value(context)?, &b.get_value(context)?)?,
                std::cmp::Ordering::Less
            ))),
            Expression::If(i, t, el) => {
                if i.get_bool(context)? {
                    t.get_value(context)
                } else {
                    el.get_value(context)
                }
            }
        }
    }
    fn get_f32(&self, context: &Context) -> Result<f32, ExpressionError> {
        self.get_value(context)?.get_f32()
    }
    fn get_i32(&self, context: &Context) -> Result<i32, ExpressionError> {
        self.get_value(context)?.get_i32()
    }
    fn get_vec2(&self, context: &Context) -> Result<Vec2, ExpressionError> {
        self.get_value(context)?.get_vec2()
    }
    fn get_bool(&self, context: &Context) -> Result<bool, ExpressionError> {
        self.get_value(context)?.get_bool()
    }
    fn get_color(&self, context: &Context) -> Result<Color32, ExpressionError> {
        self.get_value(context)?.get_color()
    }
    fn get_string(&self, context: &Context) -> Result<String, ExpressionError> {
        self.get_value(context)?.get_string()
    }
}
