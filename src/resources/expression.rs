use super::*;

pub trait ExpressionImpl {
    fn get_value(
        &self,
        e: Entity,
        s: &Query<&NodeState>,
        p: &Query<&Parent>,
    ) -> Result<VarValue, ExpressionError>;
    fn get_f32(
        &self,
        e: Entity,
        s: &Query<&NodeState>,
        p: &Query<&Parent>,
    ) -> Result<f32, ExpressionError>;
    fn get_i32(
        &self,
        e: Entity,
        s: &Query<&NodeState>,
        p: &Query<&Parent>,
    ) -> Result<i32, ExpressionError>;
    fn get_bool(
        &self,
        e: Entity,
        s: &Query<&NodeState>,
        p: &Query<&Parent>,
    ) -> Result<bool, ExpressionError>;
}

impl ExpressionImpl for Expression {
    fn get_value(
        &self,
        e: Entity,
        s: &Query<&NodeState>,
        p: &Query<&Parent>,
    ) -> Result<VarValue, ExpressionError> {
        match self {
            Expression::One => Ok(1.into()),
            Expression::Zero => Ok(0.into()),
            Expression::Var(var) => NodeState::get_var_e(*var, e, s, p).to_e(),
            Expression::Value(v) => Ok(v.clone()),
            Expression::S(s) => Ok(s.clone().into()),
            Expression::GT => Ok(gt().play_head().into()),
            Expression::Sin(x) => Ok(x.get_f32(e, s, p)?.sin().into()),
            Expression::Cos(x) => Ok(x.get_f32(e, s, p)?.cos().into()),
            Expression::Even(x) => Ok((x.get_i32(e, s, p)? % 2 == 0).into()),
            Expression::Abs(x) => x.get_value(e, s, p)?.abs(),
            Expression::Floor(x) => Ok(x.get_f32(e, s, p)?.floor().into()),
            Expression::Ceil(x) => Ok(x.get_f32(e, s, p)?.ceil().into()),
            Expression::Fract(x) => Ok(x.get_f32(e, s, p)?.fract().into()),
            Expression::Sqr(x) => Ok({
                let x = x.get_f32(e, s, p)?;
                (x * x).into()
            }),
            Expression::Sum(a, b) => a.get_value(e, s, p)?.add(&b.get_value(e, s, p)?),
            Expression::Sub(a, b) => a.get_value(e, s, p)?.sub(&b.get_value(e, s, p)?),
            Expression::Mul(a, b) => a.get_value(e, s, p)?.mul(&b.get_value(e, s, p)?),
            Expression::Div(a, b) => a.get_value(e, s, p)?.div(&b.get_value(e, s, p)?),
            Expression::Max(a, b) => a.get_value(e, s, p)?.max(&b.get_value(e, s, p)?),
            Expression::Min(a, b) => a.get_value(e, s, p)?.min(&b.get_value(e, s, p)?),
            Expression::Mod(a, b) => Ok((a.get_i32(e, s, p)? % b.get_i32(e, s, p)?).into()),
            Expression::And(a, b) => Ok((a.get_bool(e, s, p)? && b.get_bool(e, s, p)?).into()),
            Expression::Or(a, b) => Ok((a.get_bool(e, s, p)? || b.get_bool(e, s, p)?).into()),
            Expression::Equals(a, b) => Ok((a.get_value(e, s, p)? == b.get_value(e, s, p)?).into()),
            Expression::GreaterThen(a, b) => Ok(VarValue::bool(matches!(
                VarValue::compare(&a.get_value(e, s, p)?, &b.get_value(e, s, p)?)?,
                std::cmp::Ordering::Greater
            ))),
            Expression::LessThen(a, b) => Ok(VarValue::bool(matches!(
                VarValue::compare(&a.get_value(e, s, p)?, &b.get_value(e, s, p)?)?,
                std::cmp::Ordering::Less
            ))),
        }
    }
    fn get_f32(
        &self,
        e: Entity,
        s: &Query<&NodeState>,
        p: &Query<&Parent>,
    ) -> Result<f32, ExpressionError> {
        self.get_value(e, s, p)?.get_f32()
    }
    fn get_i32(
        &self,
        e: Entity,
        s: &Query<&NodeState>,
        p: &Query<&Parent>,
    ) -> Result<i32, ExpressionError> {
        self.get_value(e, s, p)?.get_i32()
    }
    fn get_bool(
        &self,
        e: Entity,
        s: &Query<&NodeState>,
        p: &Query<&Parent>,
    ) -> Result<bool, ExpressionError> {
        self.get_value(e, s, p)?.get_bool()
    }
}
