use std::f32::consts::PI;

use mlua::Lua;
use rand::seq::IndexedRandom;
use rand_chacha::rand_core::SeedableRng;

use super::*;

pub trait ExpressionImpl {
    fn get_value(&self, ctx: &mut ClientContext) -> Result<VarValue, NodeError>;
    fn get_f32(&self, ctx: &mut ClientContext) -> Result<f32, NodeError>;
    fn get_i32(&self, ctx: &mut ClientContext) -> Result<i32, NodeError>;
    fn get_vec2(&self, ctx: &mut ClientContext) -> Result<Vec2, NodeError>;
    fn get_bool(&self, ctx: &mut ClientContext) -> Result<bool, NodeError>;
    fn get_color(&self, ctx: &mut ClientContext) -> Result<Color32, NodeError>;
    fn get_string(&self, ctx: &mut ClientContext) -> Result<String, NodeError>;
    fn get_u64(&self, ctx: &mut ClientContext) -> Result<u64, NodeError>;
    fn get_u64_list(&self, ctx: &mut ClientContext) -> Result<Vec<u64>, NodeError>;
}

impl ExpressionImpl for Expression {
    fn get_value(&self, ctx: &mut ClientContext) -> Result<VarValue, NodeError> {
        match self {
            Expression::one => Ok(1.into()),
            Expression::zero => Ok(0.into()),

            Expression::pi => Ok(PI.into()),
            Expression::pi2 => Ok((PI * 2.0).into()),
            Expression::x => ctx.get_var(VarName::stax),
            Expression::owner => Ok(ctx.owner().to_not_found()?.into()),
            Expression::target => Ok(ctx.target().to_not_found()?.into()),
            Expression::var(var) => ctx.get_var(*var).or_else(|_| {
                if *var == VarName::index {
                    Ok(1.into())
                } else {
                    Err(NodeError::var_not_found(*var)).track()
                }
            }),
            Expression::owner_var(var) => ctx.owner_var(*var),
            Expression::target_var(var) => ctx.target_var(*var),
            Expression::caster_var(var) => ctx.caster_var(*var),
            Expression::status_var(var) => ctx.source().get_var(ctx.status().to_not_found()?, *var),
            Expression::var_or_zero(var) => ctx.owner_var(*var).or_else(|_| Ok(0.into())),
            Expression::state_var(x, var) => {
                let id = x.get_u64(ctx)?;
                NodeStateHistory::load(id.entity(ctx)?, ctx)?
                    .get(*var)
                    .to_e(*var)
            }
            Expression::value(v) => Ok(v.clone()),
            Expression::f32(v) | Expression::f32_slider(v) => Ok((*v).into()),
            Expression::i32(v) => Ok((*v).into()),
            Expression::bool(v) => Ok((*v).into()),
            Expression::vec2(x, y) => Ok(vec2(*x, *y).into()),
            Expression::string(s) => Ok(s.clone().into()),
            Expression::color(s) => s
                .try_c32()
                .map_err(|e| {
                    NodeError::not_supported_with_msg(
                        "Hex color parse",
                        default(),
                        format!("{e:?}"),
                    )
                })
                .map(|v| v.into()),
            Expression::gt => Ok(gt().play_head().into()),
            Expression::unit_size => Ok(UNIT_SIZE.into()),
            Expression::all_units => Ok(ctx.battle_mut()?.all_fusions().into()),
            Expression::all_ally_units => Ok(ctx
                .battle()?
                .all_allies(ctx.owner().to_not_found()?)?
                .clone()
                .into()),
            Expression::all_other_ally_units => Ok(ctx
                .battle()?
                .all_allies(ctx.owner().to_not_found()?)?
                .into_iter()
                .filter(|v| **v != ctx.owner().unwrap())
                .copied()
                .collect_vec()
                .into()),
            Expression::all_enemy_units => Ok(ctx
                .battle()?
                .all_enemies(ctx.owner().to_not_found()?)?
                .clone()
                .into()),
            Expression::adjacent_ally_units => {
                let owner = ctx.owner().to_not_found()?;
                let bs = ctx.battle()?;
                Ok(bs
                    .offset_unit(owner, -1)
                    .into_iter()
                    .chain(bs.offset_unit(owner, 1))
                    .collect_vec()
                    .into())
            }
            Expression::adjacent_front => ctx
                .battle()?
                .offset_unit(ctx.owner().to_not_found()?, -1)
                .map(|e| e.into())
                .to_custom_e("No front unit found"),
            Expression::adjacent_back => ctx
                .battle()?
                .offset_unit(ctx.owner().to_not_found()?, 1)
                .map(|e| e.into())
                .to_custom_e("No back unit found"),
            Expression::sin(x) => Ok(x.get_f32(ctx)?.sin().into()),
            Expression::cos(x) => Ok(x.get_f32(ctx)?.cos().into()),
            Expression::even(x) => Ok((x.get_i32(ctx)? % 2 == 0).into()),
            Expression::abs(x) => x.get_value(ctx)?.abs(),
            Expression::floor(x) => Ok(x.get_f32(ctx)?.floor().into()),
            Expression::ceil(x) => Ok(x.get_f32(ctx)?.ceil().into()),
            Expression::fract(x) => Ok(x.get_f32(ctx)?.fract().into()),
            Expression::dbg(x) => Ok(dbg!(x.get_value(ctx))?),
            Expression::sqr(x) => Ok({
                let x = x.get_f32(ctx)?;
                (x * x).into()
            }),
            Expression::unit_vec(x) => {
                let x = x.get_f32(ctx)?;
                let x = vec2(x.cos(), x.sin());
                Ok(x.into())
            }
            Expression::to_f32(x) => Ok(x.get_f32(ctx)?.into()),
            Expression::rand(x) => {
                let x = x.get_value(ctx)?;
                let mut hasher = DefaultHasher::new();
                x.hash(&mut hasher);
                let mut rng = ChaCha8Rng::seed_from_u64(hasher.finish());
                Ok(rng.random_range(0.0..1.0).into())
            }
            Expression::random_unit(x) => x
                .get_u64_list(ctx)?
                .choose(ctx.rng()?)
                .map(|id| VarValue::u64(*id))
                .to_custom_e("No units found"),
            Expression::neg(x) => x.get_value(ctx)?.neg(),
            Expression::str_macro(s, v) => {
                let s = s.get_string(ctx)?;
                let v = v.get_string(ctx)?;
                Ok(s.replace("%s", &v).into())
            }
            Expression::vec2_ee(a, b) => Ok(vec2(a.get_f32(ctx)?, b.get_f32(ctx)?).into()),
            Expression::sum(a, b) => a.get_value(ctx)?.add(&b.get_value(ctx)?),
            Expression::sub(a, b) => a.get_value(ctx)?.sub(&b.get_value(ctx)?),
            Expression::mul(a, b) => a.get_value(ctx)?.mul(&b.get_value(ctx)?),
            Expression::div(a, b) => a.get_value(ctx)?.div(&b.get_value(ctx)?),
            Expression::max(a, b) => a.get_value(ctx)?.max(&b.get_value(ctx)?),
            Expression::min(a, b) => a.get_value(ctx)?.min(&b.get_value(ctx)?),
            Expression::r#mod(a, b) => Ok((a.get_i32(ctx)? % b.get_i32(ctx)?).into()),
            Expression::and(a, b) => Ok((a.get_bool(ctx)? && b.get_bool(ctx)?).into()),
            Expression::or(a, b) => Ok((a.get_bool(ctx)? || b.get_bool(ctx)?).into()),
            Expression::equals(a, b) => Ok((a.get_value(ctx)? == b.get_value(ctx)?).into()),
            Expression::greater_then(a, b) => Ok(VarValue::bool(matches!(
                VarValue::compare(&a.get_value(ctx)?, &b.get_value(ctx)?)?,
                std::cmp::Ordering::Greater
            ))),
            Expression::less_then(a, b) => Ok(VarValue::bool(matches!(
                VarValue::compare(&a.get_value(ctx)?, &b.get_value(ctx)?)?,
                std::cmp::Ordering::Less
            ))),
            Expression::fallback(v, fb) => {
                if let Ok(v) = v.get_value(ctx) {
                    Ok(v)
                } else {
                    fb.get_value(ctx)
                }
            }
            Expression::oklch(l, c, h) => Ok(Color::lch(
                l.get_f32(ctx)? * 1.5,
                c.get_f32(ctx)? * 1.5,
                h.get_f32(ctx)? * 360.0,
            )
            .c32()
            .into()),
            Expression::r#if(i, t, el) => {
                if i.get_bool(ctx)? {
                    t.get_value(ctx)
                } else {
                    el.get_value(ctx)
                }
            }
            Expression::lua_f32(code) => {
                let lua = Lua::new();
                let v: f32 = lua
                    .load(code)
                    .eval::<f32>()
                    .map_err(|e| format!("lua error: {e}"))?;
                Ok(v.into())
            }
            Expression::lua_i32(code) => {
                let lua = Lua::new();
                let v: i32 = lua
                    .load(code)
                    .eval::<i32>()
                    .map_err(|e| format!("lua error: {e}"))?;
                Ok(v.into())
            }
        }
    }
    fn get_f32(&self, ctx: &mut ClientContext) -> Result<f32, NodeError> {
        self.get_value(ctx)?.get_f32()
    }
    fn get_i32(&self, ctx: &mut ClientContext) -> Result<i32, NodeError> {
        self.get_value(ctx)?.get_i32()
    }
    fn get_vec2(&self, ctx: &mut ClientContext) -> Result<Vec2, NodeError> {
        self.get_value(ctx)?.get_vec2()
    }
    fn get_bool(&self, ctx: &mut ClientContext) -> Result<bool, NodeError> {
        self.get_value(ctx)?.get_bool()
    }
    fn get_color(&self, ctx: &mut ClientContext) -> Result<Color32, NodeError> {
        self.get_value(ctx)?.get_color()
    }
    fn get_string(&self, ctx: &mut ClientContext) -> Result<String, NodeError> {
        self.get_value(ctx)?.get_string()
    }
    fn get_u64(&self, ctx: &mut ClientContext) -> Result<u64, NodeError> {
        self.get_value(ctx)?.get_u64()
    }
    fn get_u64_list(&self, ctx: &mut ClientContext) -> Result<Vec<u64>, NodeError> {
        self.get_value(ctx)?.get_u64_list()
    }
}
