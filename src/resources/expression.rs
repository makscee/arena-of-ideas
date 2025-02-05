use std::f32::consts::PI;

use rand::{seq::SliceRandom, thread_rng};

use super::*;

pub trait ExpressionImpl {
    fn get_value(&self, context: &Context) -> Result<VarValue, ExpressionError>;
    fn get_f32(&self, context: &Context) -> Result<f32, ExpressionError>;
    fn get_i32(&self, context: &Context) -> Result<i32, ExpressionError>;
    fn get_vec2(&self, context: &Context) -> Result<Vec2, ExpressionError>;
    fn get_bool(&self, context: &Context) -> Result<bool, ExpressionError>;
    fn get_color(&self, context: &Context) -> Result<Color32, ExpressionError>;
    fn get_string(&self, context: &Context) -> Result<String, ExpressionError>;
    fn get_entity(&self, context: &Context) -> Result<Entity, ExpressionError>;
    fn get_entity_list(&self, context: &Context) -> Result<Vec<Entity>, ExpressionError>;
}

impl ExpressionImpl for Expression {
    fn get_value(&self, context: &Context) -> Result<VarValue, ExpressionError> {
        match self {
            Expression::One => Ok(1.into()),
            Expression::Zero => Ok(0.into()),
            Expression::PI => Ok(PI.into()),
            Expression::PI2 => Ok((PI * 2.0).into()),
            Expression::Owner => Ok(context.get_owner()?.to_value()),
            Expression::Target => Ok(context.get_target()?.to_value()),
            Expression::Var(var) => {
                let v = context.get_var(*var);
                if v.is_err() && *var == VarName::index {
                    Ok(1.into())
                } else {
                    v
                }
            }
            Expression::StateVar(x, var) => context
                .get_state(x.get_entity(context)?)?
                .get(*var)
                .to_e_var(*var),
            Expression::V(v) => Ok(v.clone()),
            Expression::F(v) | Expression::FSlider(v) => Ok((*v).into()),
            Expression::I(v) => Ok((*v).into()),
            Expression::B(v) => Ok((*v).into()),
            Expression::V2(x, y) => Ok(vec2(*x, *y).into()),
            Expression::S(s) => Ok(s.clone().into()),
            Expression::C(s) => Color32::from_hex(s)
                .map_err(|e| ExpressionError::OperationNotSupported {
                    values: default(),
                    op: "Hex color parse",
                    msg: Some(format!("{e:?}")),
                })
                .map(|v| v.into()),
            Expression::GT => Ok(gt().play_head().into()),
            Expression::UnitSize => Ok(UNIT_SIZE.into()),
            Expression::AllUnits => Ok(context.get_all_units().into()),
            Expression::AllAllyUnits => Ok(context.all_allies(context.get_owner()?).into()),
            Expression::AllOtherAllyUnits => Ok(context
                .all_allies(context.get_owner()?)
                .into_iter()
                .filter(|v| v.get_entity().unwrap() != context.get_owner().unwrap())
                .collect_vec()
                .into()),
            Expression::AllEnemyUnits => Ok(context.all_enemeis(context.get_owner()?).into()),
            Expression::AdjacentAllyUnits => {
                Ok(context.adjacent_allies(context.get_owner()?).into())
            }
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
            Expression::UnitVec(x) => {
                let x = x.get_f32(context)?;
                let x = vec2(x.cos(), x.sin());
                Ok(x.into())
            }
            Expression::ToF(x) => Ok(x.get_f32(context)?.into()),
            Expression::Rand(x) => {
                let x = x.get_value(context)?;
                let mut hasher = DefaultHasher::new();
                x.hash(&mut hasher);
                let mut rng = ChaCha8Rng::seed_from_u64(hasher.finish());
                Ok(rng.gen_range(0.0..1.0).into())
            }
            Expression::RandomUnit(x) => x
                .get_entity_list(context)?
                .choose(&mut thread_rng())
                .map(|e| e.to_value())
                .to_e("No units found"),
            Expression::Macro(s, v) => {
                let s = s.get_string(context)?;
                let v = v.get_string(context)?;
                Ok(s.replace("%s", &v).into())
            }
            Expression::V2EE(a, b) => Ok(vec2(a.get_f32(context)?, b.get_f32(context)?).into()),
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
            Expression::Fallback(v, fb) => {
                if let Ok(v) = v.get_value(context) {
                    Ok(v)
                } else {
                    fb.get_value(context)
                }
            }
            Expression::Oklch(l, c, h) => Ok(Color::lch(
                l.get_f32(context)? * 1.5,
                c.get_f32(context)? * 1.5,
                h.get_f32(context)? * 360.0,
            )
            .c32()
            .into()),
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
    fn get_entity(&self, context: &Context) -> Result<Entity, ExpressionError> {
        self.get_value(context)?.get_entity()
    }
    fn get_entity_list(&self, context: &Context) -> Result<Vec<Entity>, ExpressionError> {
        self.get_value(context)?.get_entity_list()
    }
}
