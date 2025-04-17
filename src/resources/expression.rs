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
            Expression::one => Ok(1.into()),
            Expression::zero => Ok(0.into()),
            Expression::pi => Ok(PI.into()),
            Expression::pi2 => Ok((PI * 2.0).into()),
            Expression::owner => Ok(context.get_owner()?.to_value()),
            Expression::target => Ok(context.get_target()?.to_value()),
            Expression::var(var) => {
                let v = context.get_var(*var);
                if v.is_err() && *var == VarName::index {
                    Ok(1.into())
                } else {
                    v
                }
            }
            Expression::state_var(x, var) => context
                .get_state(x.get_entity(context)?)?
                .get(*var)
                .to_e_var(*var),
            Expression::value(v) => Ok(v.clone()),
            Expression::f32(v) | Expression::f32_slider(v) => Ok((*v).into()),
            Expression::i32(v) => Ok((*v).into()),
            Expression::bool(v) => Ok((*v).into()),
            Expression::vec2(x, y) => Ok(vec2(*x, *y).into()),
            Expression::string(s) => Ok(s.clone().into()),
            Expression::color(s) => s
                .try_c32()
                .map_err(|e| ExpressionError::OperationNotSupported {
                    values: default(),
                    op: "Hex color parse",
                    msg: Some(format!("{e:?}")),
                })
                .map(|v| v.into()),
            Expression::gt => Ok(gt().play_head().into()),
            Expression::unit_size => Ok(UNIT_SIZE.into()),
            Expression::all_units => Ok(context.get_all_units().into()),
            Expression::all_ally_units => Ok(context.all_allies(context.get_owner()?).into()),
            Expression::all_other_ally_units => Ok(context
                .all_allies(context.get_owner()?)
                .into_iter()
                .filter(|v| v.get_entity().unwrap() != context.get_owner().unwrap())
                .collect_vec()
                .into()),
            Expression::all_enemy_units => Ok(context.all_enemies(context.get_owner()?).into()),
            Expression::adjacent_ally_units => {
                Ok(context.adjacent_allies(context.get_owner()?).into())
            }
            Expression::adjacent_front => context
                .offset_unit(context.get_owner()?, -1)
                .to_e("No front unit found"),
            Expression::adjacent_back => context
                .offset_unit(context.get_owner()?, 1)
                .to_e("No back unit found"),
            Expression::sin(x) => Ok(x.get_f32(context)?.sin().into()),
            Expression::cos(x) => Ok(x.get_f32(context)?.cos().into()),
            Expression::even(x) => Ok((x.get_i32(context)? % 2 == 0).into()),
            Expression::abs(x) => x.get_value(context)?.abs(),
            Expression::floor(x) => Ok(x.get_f32(context)?.floor().into()),
            Expression::ceil(x) => Ok(x.get_f32(context)?.ceil().into()),
            Expression::fract(x) => Ok(x.get_f32(context)?.fract().into()),
            Expression::sqr(x) => Ok({
                let x = x.get_f32(context)?;
                (x * x).into()
            }),
            Expression::unit_vec(x) => {
                let x = x.get_f32(context)?;
                let x = vec2(x.cos(), x.sin());
                Ok(x.into())
            }
            Expression::to_f32(x) => Ok(x.get_f32(context)?.into()),
            Expression::rand(x) => {
                let x = x.get_value(context)?;
                let mut hasher = DefaultHasher::new();
                x.hash(&mut hasher);
                let mut rng = ChaCha8Rng::seed_from_u64(hasher.finish());
                Ok(rng.gen_range(0.0..1.0).into())
            }
            Expression::random_unit(x) => x
                .get_entity_list(context)?
                .choose(&mut thread_rng())
                .map(|e| e.to_value())
                .to_e("No units found"),
            Expression::str_macro(s, v) => {
                let s = s.get_string(context)?;
                let v = v.get_string(context)?;
                Ok(s.replace("%s", &v).into())
            }
            Expression::vec2_ee(a, b) => Ok(vec2(a.get_f32(context)?, b.get_f32(context)?).into()),
            Expression::sum(a, b) => a.get_value(context)?.add(&b.get_value(context)?),
            Expression::sub(a, b) => a.get_value(context)?.sub(&b.get_value(context)?),
            Expression::mul(a, b) => a.get_value(context)?.mul(&b.get_value(context)?),
            Expression::div(a, b) => a.get_value(context)?.div(&b.get_value(context)?),
            Expression::max(a, b) => a.get_value(context)?.max(&b.get_value(context)?),
            Expression::min(a, b) => a.get_value(context)?.min(&b.get_value(context)?),
            Expression::r#mod(a, b) => Ok((a.get_i32(context)? % b.get_i32(context)?).into()),
            Expression::and(a, b) => Ok((a.get_bool(context)? && b.get_bool(context)?).into()),
            Expression::or(a, b) => Ok((a.get_bool(context)? || b.get_bool(context)?).into()),
            Expression::equals(a, b) => Ok((a.get_value(context)? == b.get_value(context)?).into()),
            Expression::greater_then(a, b) => Ok(VarValue::bool(matches!(
                VarValue::compare(&a.get_value(context)?, &b.get_value(context)?)?,
                std::cmp::Ordering::Greater
            ))),
            Expression::less_then(a, b) => Ok(VarValue::bool(matches!(
                VarValue::compare(&a.get_value(context)?, &b.get_value(context)?)?,
                std::cmp::Ordering::Less
            ))),
            Expression::fallback(v, fb) => {
                if let Ok(v) = v.get_value(context) {
                    Ok(v)
                } else {
                    fb.get_value(context)
                }
            }
            Expression::oklch(l, c, h) => Ok(Color::lch(
                l.get_f32(context)? * 1.5,
                c.get_f32(context)? * 1.5,
                h.get_f32(context)? * 360.0,
            )
            .c32()
            .into()),
            Expression::r#if(i, t, el) => {
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
