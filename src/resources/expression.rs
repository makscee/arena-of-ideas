use std::f32::consts::PI;

use rand::{seq::IteratorRandom, thread_rng};

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, EnumIter, AsRefStr)]
pub enum Expression {
    #[default]
    Zero,
    Dbg(Box<Expression>),

    OppositeFaction,
    SlotPosition,
    GT,
    Beat,
    PI,
    PI2,
    Age,
    Index,

    Owner,
    Caster,
    Target,

    RandomAlly,
    RandomEnemy,
    RandomAdjacentUnit,
    AllAllyUnits,
    AllEnemyUnits,
    AllUnits,
    AllOtherUnits,
    AdjacentUnits,

    Value(VarValue),
    Context(VarName),
    OwnerState(VarName),
    TargetState(VarName),
    CasterState(VarName),
    OwnerStateLast(VarName),
    TargetStateLast(VarName),
    CasterStateLast(VarName),
    AbilityContext(String, VarName),
    AbilityState(String, VarName),
    StatusCharges(String),
    HexColor(String),
    F(f32),
    I(i32),
    B(bool),
    S(String),
    V2(f32, f32),

    Vec2E(Box<Expression>),
    UnitVec(Box<Expression>),
    Sin(Box<Expression>),
    Cos(Box<Expression>),
    Even(Box<Expression>),
    Abs(Box<Expression>),
    FactionCount(Box<Expression>),
    SlotUnit(Box<Expression>),

    Vec2EE(Box<Expression>, Box<Expression>),
    Sum(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Mod(Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),
    Equals(Box<Expression>, Box<Expression>),
    GreaterThen(Box<Expression>, Box<Expression>),
    LessThen(Box<Expression>, Box<Expression>),

    If(Box<Expression>, Box<Expression>, Box<Expression>),

    WithVar(VarName, Box<Expression>, Box<Expression>),
}

impl Expression {
    pub fn get_value(&self, context: &Context, world: &mut World) -> Result<VarValue> {
        match self {
            Expression::Zero => Ok(VarValue::Int(0)),
            Expression::Dbg(e) => dbg!(e.get_value(context, world)),
            Expression::Value(v) => Ok(v.clone()),
            Expression::Context(var) => context.get_var(*var, world),
            Expression::OwnerState(var) => {
                VarState::find_value_at(context.owner(), *var, gt().play_head(), world)
            }
            Expression::TargetState(var) => {
                VarState::find_value_at(context.get_target()?, *var, gt().play_head(), world)
            }
            Expression::CasterState(var) => {
                VarState::find_value_at(context.get_caster()?, *var, gt().play_head(), world)
            }
            Expression::OwnerStateLast(var) => {
                VarState::find_value_last(context.owner(), *var, world)
            }
            Expression::TargetStateLast(var) => {
                VarState::find_value_last(context.get_target()?, *var, world)
            }
            Expression::CasterStateLast(var) => {
                VarState::find_value_last(context.get_caster()?, *var, world)
            }
            Expression::AbilityContext(ability, var) => context.get_ability_var(ability, *var),
            Expression::AbilityState(ability, var) => {
                Ok(
                    TeamPlugin::get_ability_state(ability, context.get_faction(world)?, world)
                        .and_then(|s| s.get_value_at(*var, gt().play_head()).ok())
                        .unwrap_or_else(|| GameAssets::ability_default(&ability, *var, world)),
                )
            }
            Expression::WithVar(var, value, e) => e.get_value(
                context
                    .clone()
                    .set_var(*var, value.get_value(context, world)?),
                world,
            ),
            Expression::StatusCharges(name) => {
                Ok(Status::get_charges(name, context.owner(), world)?.into())
            }
            Expression::HexColor(s) => Ok(VarValue::Color(Color::hex(s)?)),
            Expression::Sin(v) => Ok(v.get_float(context, world)?.sin().into()),
            Expression::Cos(v) => Ok(v.get_float(context, world)?.cos().into()),
            Expression::Abs(v) => v.get_value(context, world)?.abs(),
            Expression::Even(v) => Ok((v.get_int(context, world)? % 2 == 0).into()),
            Expression::FactionCount(v) => Ok(UnitPlugin::collect_faction(
                v.get_faction(context, world)?,
                world,
            )
            .len()
            .into()),
            Expression::Sum(a, b) => Ok(VarValue::sum(
                &a.get_value(context, world)?,
                &b.get_value(context, world)?,
            )?),
            Expression::Sub(a, b) => Ok(VarValue::sub(
                &a.get_value(context, world)?,
                &b.get_value(context, world)?,
            )?),
            Expression::Mul(a, b) => Ok(VarValue::mul(
                &a.get_value(context, world)?,
                &b.get_value(context, world)?,
            )?),
            Expression::Div(a, b) => Ok(VarValue::div(
                &a.get_value(context, world)?,
                &b.get_value(context, world)?,
            )?),
            Expression::Mod(a, b) => {
                Ok((a.get_int(context, world)? % b.get_int(context, world)?).into())
            }
            Expression::And(a, b) => {
                Ok((a.get_bool(context, world)? && b.get_bool(context, world)?).into())
            }
            Expression::Or(a, b) => {
                Ok((a.get_bool(context, world)? || b.get_bool(context, world)?).into())
            }
            Expression::Equals(a, b) => Ok(a
                .get_value(context, world)?
                .eq(&b.get_value(context, world)?)
                .into()),
            Expression::GreaterThen(a, b) => Ok(VarValue::Bool(matches!(
                VarValue::compare(&a.get_value(context, world)?, &b.get_value(context, world)?,)?,
                std::cmp::Ordering::Greater
            ))),
            Expression::LessThen(a, b) => Ok(VarValue::Bool(matches!(
                VarValue::compare(&a.get_value(context, world)?, &b.get_value(context, world)?,)?,
                std::cmp::Ordering::Less
            ))),
            Expression::OppositeFaction => Ok(VarValue::Faction(
                context
                    .get_var(VarName::Faction, world)?
                    .get_faction()?
                    .opposite(),
            )),
            Expression::Owner => Ok(VarValue::Entity(context.owner())),
            Expression::Caster => Ok(VarValue::Entity(context.get_caster()?)),
            Expression::Target => Ok(VarValue::Entity(context.get_target()?)),
            Expression::SlotUnit(index) => Ok(VarValue::Entity(
                UnitPlugin::find_unit(
                    context.get_var(VarName::Faction, world)?.get_faction()?,
                    index.get_int(context, world)?,
                    world,
                )
                .context("No unit in slot")?,
            )),
            Expression::If(cond, th, el) => {
                if cond.get_bool(context, world)? {
                    th.get_value(context, world)
                } else {
                    el.get_value(context, world)
                }
            }
            Expression::UnitVec(x) => {
                let x = x.get_float(context, world)?;
                let x = vec2(x.cos(), x.sin());
                Ok(x.into())
            }
            Expression::Vec2E(e) => {
                let v = e.get_float(context, world)?;
                Ok(VarValue::Vec2(vec2(v, v)))
            }
            Expression::Vec2EE(x, y) => Ok(VarValue::Vec2(vec2(
                x.get_float(context, world)?,
                y.get_float(context, world)?,
            ))),
            Expression::SlotPosition => Ok(VarValue::Vec2(UnitPlugin::get_entity_slot_position(
                context.owner(),
                world,
            )?)),
            Expression::F(v) => Ok((*v).into()),
            Expression::I(v) => Ok((*v).into()),
            Expression::B(v) => Ok((*v).into()),
            Expression::S(v) => Ok((v.clone()).into()),
            Expression::V2(x, y) => Ok(vec2(*x, *y).into()),
            Expression::GT => Ok(gt().play_head().into()),
            Expression::Beat => Ok(gt().play_head().sin().into()),

            Expression::PI => Ok(VarValue::Float(PI)),
            Expression::PI2 => Ok(VarValue::Float(PI * 2.0)),
            Expression::RandomAlly => {
                UnitPlugin::collect_faction(context.get_faction(world)?, world)
                    .into_iter()
                    .filter(|e| !context.owner().eq(e))
                    .choose(&mut thread_rng())
                    .context("No other units found")
                    .map(|v| v.into())
            }
            Expression::RandomEnemy => Self::RandomAlly.get_value(
                &context.clone().set_var(
                    VarName::Faction,
                    context.get_faction(world)?.opposite().into(),
                ),
                world,
            ),
            Expression::AdjacentUnits => {
                let own_slot = context.get_var(VarName::Slot, world)?.get_int()?;
                let faction = context.get_var(VarName::Faction, world)?.get_faction()?;
                let mut left: (i32, Option<Entity>) = (-i32::MAX, None);
                let mut right: (i32, Option<Entity>) = (i32::MAX, None);
                for unit in UnitPlugin::collect_faction(faction, world) {
                    let state = VarState::get(unit, world);
                    let slot = state.get_int(VarName::Slot)?;
                    let delta = slot - own_slot;
                    if delta == 0 {
                        continue;
                    }
                    if delta < 0 && left.0 < delta {
                        left.0 = delta;
                        left.1 = Some(unit);
                    }
                    if delta > 0 && right.0 > delta {
                        right.0 = delta;
                        right.1 = Some(unit);
                    }
                }
                Ok(VarValue::List(
                    left.1
                        .into_iter()
                        .chain(right.1.into_iter())
                        .map(|e| e.into())
                        .collect_vec(),
                ))
            }
            Expression::RandomAdjacentUnit => Ok(Self::AdjacentUnits
                .get_value(context, world)?
                .get_entity_list()?
                .into_iter()
                .choose(&mut thread_rng())
                .context("No adjacent units")?
                .into()),
            Expression::AllAllyUnits => Ok(VarValue::List(
                UnitPlugin::collect_faction(context.get_faction(world)?, world)
                    .into_iter()
                    .map(|e| e.into())
                    .collect_vec(),
            )),
            Expression::AllEnemyUnits => Ok(VarValue::List(
                UnitPlugin::collect_faction(context.get_faction(world)?.opposite(), world)
                    .into_iter()
                    .map(|e| e.into())
                    .collect_vec(),
            )),
            Expression::AllUnits => Ok(VarValue::List(
                UnitPlugin::collect_alive(world)
                    .into_iter()
                    .map(|e| e.into())
                    .collect_vec(),
            )),
            Expression::AllOtherUnits => Ok(VarValue::List(
                UnitPlugin::collect_alive(world)
                    .into_iter()
                    .filter_map(|e| match e.eq(&context.owner()) {
                        true => None,
                        false => Some(e.into()),
                    })
                    .collect_vec(),
            )),
            Expression::Age => {
                Ok((gt().play_head() - VarState::get(context.owner(), world).birth()).into())
            }
            Expression::Index => Expression::Context(VarName::Index).get_value(context, world),
        }
    }
    pub fn get_float(&self, context: &Context, world: &mut World) -> Result<f32> {
        self.get_value(context, world)?.get_float()
    }
    pub fn get_int(&self, context: &Context, world: &mut World) -> Result<i32> {
        self.get_value(context, world)?.get_int()
    }
    pub fn get_vec2(&self, context: &Context, world: &mut World) -> Result<Vec2> {
        self.get_value(context, world)?.get_vec2()
    }
    pub fn get_bool(&self, context: &Context, world: &mut World) -> Result<bool> {
        self.get_value(context, world)?.get_bool()
    }
    pub fn get_string(&self, context: &Context, world: &mut World) -> Result<String> {
        self.get_value(context, world)?.get_string()
    }
    pub fn get_entity(&self, context: &Context, world: &mut World) -> Result<Entity> {
        self.get_value(context, world)?.get_entity()
    }
    pub fn get_faction(&self, context: &Context, world: &mut World) -> Result<Faction> {
        self.get_value(context, world)?.get_faction()
    }
    pub fn get_color(&self, context: &Context, world: &mut World) -> Result<Color> {
        self.get_value(context, world)?.get_color()
    }
}

impl ToCstr for Expression {
    fn cstr(&self) -> Cstr {
        let mut s = self.as_ref().to_case(Case::Lower).cstr_c(VISIBLE_BRIGHT);
        match self {
            Expression::Value(v) => {
                s.push(
                    v.cstr()
                        .wrap(("(".cstr(), ")".cstr()))
                        .color(VISIBLE_BRIGHT)
                        .take(),
                );
            }
            Expression::OwnerState(v)
            | Expression::TargetState(v)
            | Expression::CasterState(v)
            | Expression::OwnerStateLast(v)
            | Expression::TargetStateLast(v)
            | Expression::CasterStateLast(v)
            | Expression::Context(v) => {
                s.push(
                    v.cstr()
                        .wrap(("(".cstr(), ")".cstr()))
                        .color(VISIBLE_BRIGHT)
                        .take(),
                );
            }
            Expression::AbilityContext(name, v) | Expression::AbilityState(name, v) => {
                s.push(
                    name.cstr_cs(name_color(name), CstrStyle::Bold)
                        .push(format!(", {v}").cstr())
                        .wrap(("(".cstr(), ")".cstr()))
                        .take(),
                );
            }
            Expression::StatusCharges(v) => {
                s.push(
                    v.cstr_cs(name_color(v), CstrStyle::Bold)
                        .wrap(("(".cstr(), ")".cstr()))
                        .take(),
                );
            }
            Expression::HexColor(v) => {
                s.push(v.cstr().wrap(("(".cstr(), ")".cstr())).take());
            }
            Expression::F(v) => {
                s.push(v.to_string().cstr().wrap(("(".cstr(), ")".cstr())).take());
            }
            Expression::I(v) => {
                s.push(v.to_string().cstr().wrap(("(".cstr(), ")".cstr())).take());
            }
            Expression::B(v) => {
                s.push(v.to_string().cstr().wrap(("(".cstr(), ")".cstr())).take());
            }
            Expression::S(v) => {
                s.push(v.cstr().wrap(("(\"".cstr(), "\")".cstr())).take());
            }
            Expression::V2(x, y) => {
                s.push(
                    format!("{x}, {y}")
                        .cstr()
                        .wrap(("(".cstr(), ")".cstr()))
                        .take(),
                );
            }
            Expression::Vec2E(v)
            | Expression::Dbg(v)
            | Expression::UnitVec(v)
            | Expression::Sin(v)
            | Expression::Cos(v)
            | Expression::Abs(v)
            | Expression::Even(v)
            | Expression::FactionCount(v)
            | Expression::SlotUnit(v) => {
                s.push(v.cstr().wrap(("(".cstr(), ")".cstr())).take());
            }
            Expression::Vec2EE(a, b)
            | Expression::Sum(a, b)
            | Expression::Sub(a, b)
            | Expression::Mul(a, b)
            | Expression::Div(a, b)
            | Expression::Mod(a, b)
            | Expression::And(a, b)
            | Expression::Or(a, b)
            | Expression::Equals(a, b)
            | Expression::GreaterThen(a, b)
            | Expression::LessThen(a, b) => {
                s.push(
                    a.cstr()
                        .push(", ".cstr())
                        .push(b.cstr())
                        .wrap(("(".cstr(), ")".cstr()))
                        .take(),
                );
            }
            Expression::If(i, t, e) => {
                s.push(
                    format!("if ")
                        .cstr()
                        .push(i.cstr().wrap(("(".cstr(), ")".cstr())).take())
                        .take(),
                )
                .push(
                    format!("then ")
                        .cstr()
                        .push(t.cstr().wrap(("(".cstr(), ")".cstr())).take())
                        .take(),
                )
                .push(
                    format!("else ")
                        .cstr()
                        .push(e.cstr().wrap(("(".cstr(), ")".cstr())).take())
                        .take(),
                );
            }
            Expression::WithVar(var, val, e) => {
                s.push(
                    var.cstr()
                        .push(val.cstr())
                        .push(e.cstr())
                        .join(&", ".cstr())
                        .wrap(("(".cstr(), ")".cstr()))
                        .take(),
                );
            }

            Expression::Zero
            | Expression::OppositeFaction
            | Expression::SlotPosition
            | Expression::GT
            | Expression::Beat
            | Expression::PI
            | Expression::PI2
            | Expression::Age
            | Expression::Index
            | Expression::Owner
            | Expression::Caster
            | Expression::Target
            | Expression::RandomAlly
            | Expression::RandomEnemy
            | Expression::RandomAdjacentUnit
            | Expression::AllAllyUnits
            | Expression::AllEnemyUnits
            | Expression::AllUnits
            | Expression::AllOtherUnits
            | Expression::AdjacentUnits => {}
        }
        s
    }
}
