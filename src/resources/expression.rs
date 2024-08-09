use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, EnumIter, AsRefStr)]
pub enum Expression {
    #[default]
    Zero,
    Dbg(Box<Expression>),
    Ctx(Box<Expression>),

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
    Status,

    RandomUnit(Box<Expression>),
    RandomUnitSubset(Box<Expression>, Box<Expression>),
    MaxUnit(Box<Expression>, Box<Expression>),

    AllAllyUnits,
    AllEnemyUnits,
    AllUnits,
    AllOtherUnits,
    AdjacentUnits,

    FilterStatusUnits(String, Box<Expression>),

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

    ToI(Box<Expression>),
    Vec2E(Box<Expression>),
    UnitVec(Box<Expression>),
    Sin(Box<Expression>),
    Cos(Box<Expression>),
    Even(Box<Expression>),
    Abs(Box<Expression>),
    Floor(Box<Expression>),
    Ceil(Box<Expression>),
    Fract(Box<Expression>),
    SlotUnit(Box<Expression>),
    RandomF(Box<Expression>),
    ListCount(Box<Expression>),

    Vec2EE(Box<Expression>, Box<Expression>),
    Sum(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Max(Box<Expression>, Box<Expression>),
    Min(Box<Expression>, Box<Expression>),
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
            Expression::Ctx(e) => {
                dbg!(context);
                dbg!(e.get_value(context, world))
            }
            Expression::Value(v) => Ok(v.clone()),
            Expression::Context(var) => context.get_value(*var, world),
            Expression::OwnerState(var) => {
                Context::new_play(context.owner()).get_value(*var, world)
            }
            Expression::TargetState(var) => {
                Context::new_play(context.get_target()?).get_value(*var, world)
            }
            Expression::CasterState(var) => {
                Context::new_play(context.get_caster()?).get_value(*var, world)
            }
            Expression::OwnerStateLast(var) => Context::new(context.owner()).get_value(*var, world),
            Expression::TargetStateLast(var) => {
                Context::new(context.get_target()?).get_value(*var, world)
            }
            Expression::CasterStateLast(var) => {
                Context::new(context.get_caster()?).get_value(*var, world)
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
            Expression::ListCount(v) => Ok(v.get_value(context, world)?.get_list()?.len().into()),
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
            Expression::Max(a, b) => Ok(VarValue::max(
                &a.get_value(context, world)?,
                &b.get_value(context, world)?,
            )?),
            Expression::Min(a, b) => Ok(VarValue::min(
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
                    .get_value(VarName::Faction, world)?
                    .get_faction()?
                    .opposite(),
            )),
            Expression::Owner => Ok(VarValue::Entity(context.owner())),
            Expression::Caster => Ok(VarValue::Entity(context.get_caster()?)),
            Expression::Target => Ok(VarValue::Entity(context.get_target()?)),
            Expression::Status => Ok(VarValue::Entity(context.get_status_entity(world)?)),
            Expression::SlotUnit(index) => Ok(VarValue::Entity(
                UnitPlugin::find_unit(
                    context.get_value(VarName::Faction, world)?.get_faction()?,
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
            Expression::ToI(v) => Ok(v.get_int(context, world).unwrap_or_default().into()),
            Expression::B(v) => Ok((*v).into()),
            Expression::S(v) => Ok((v.clone()).into()),
            Expression::V2(x, y) => Ok(vec2(*x, *y).into()),
            Expression::GT => Ok(gt().play_head().into()),
            Expression::Beat => Ok(gt().play_head().sin().into()),

            Expression::PI => Ok(VarValue::Float(PI)),
            Expression::PI2 => Ok(VarValue::Float(PI * 2.0)),
            Expression::AdjacentUnits => {
                let own_slot = context.get_value(VarName::Slot, world)?.get_int()?;
                let faction = context.get_value(VarName::Faction, world)?.get_faction()?;
                let mut left: (i32, Option<Entity>) = (-i32::MAX, None);
                let mut right: (i32, Option<Entity>) = (i32::MAX, None);
                for unit in UnitPlugin::collect_faction(faction, world) {
                    let slot = Context::new(unit).get_int(VarName::Slot, world)?;
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
            Expression::FilterStatusUnits(status, units) => {
                let units = units.get_value(context, world)?.get_entity_list()?;
                Ok(units
                    .into_iter()
                    .filter(|u| Status::get_charges(&status, *u, world).unwrap_or_default() > 0)
                    .collect_vec()
                    .into())
            }
            Expression::Age => {
                Ok((gt().play_head() - VarState::get(context.owner(), world).birth()).into())
            }
            Expression::Index => Expression::Context(VarName::Index).get_value(context, world),
            Expression::MaxUnit(value, units) => {
                let units = units.get_value(context, world)?.get_entity_list()?;
                if units.is_empty() {
                    return Err(anyhow!("No units found"));
                }
                units
                    .into_iter()
                    .max_by(|a, b| {
                        let a = value
                            .get_value(&context.clone().set_owner(*a), world)
                            .unwrap_or_default();
                        let b = value
                            .get_value(&context.clone().set_owner(*b), world)
                            .unwrap_or_default();
                        VarValue::compare(&a, &b).unwrap_or(Ordering::Equal)
                    })
                    .context("Filed to find max unit")
                    .map(|u| u.into())
            }
            Expression::RandomUnitSubset(amount, units) => {
                let units = units.get_value(context, world)?.get_entity_list()?;
                let amount = amount.get_int(context, world)? as usize;
                Ok(units
                    .into_iter()
                    .choose_multiple(&mut thread_rng(), amount)
                    .into())
            }
            Expression::RandomUnit(units) => {
                let units = units.get_value(context, world)?.get_entity_list()?;
                units
                    .into_iter()
                    .choose(&mut thread_rng())
                    .map(|u| u.into())
                    .context("No units to choose from")
            }
            Expression::RandomF(x) => {
                let x = x.get_value(context, world)?;
                let mut hasher = DefaultHasher::new();
                x.hash(&mut hasher);
                let mut rng = ChaCha8Rng::seed_from_u64(hasher.finish());
                Ok(rng.gen_range(0.0..1.0).into())
            }
            Expression::Floor(x) => Ok(x.get_float(context, world)?.floor().into()),
            Expression::Fract(x) => Ok(x.get_float(context, world)?.fract().into()),
            Expression::Ceil(x) => Ok(x.get_float(context, world)?.ceil().into()),
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
        let mut s = self.as_ref().to_case(Case::Lower).cstr_c(VISIBLE_LIGHT);
        match self {
            Expression::Value(v) => {
                s.push(
                    v.cstr()
                        .wrap(("(".cstr(), ")".cstr()))
                        .color(VISIBLE_LIGHT)
                        .take(),
                );
            }
            Expression::OwnerState(v)
            | Expression::TargetState(v)
            | Expression::CasterState(v)
            | Expression::OwnerStateLast(v)
            | Expression::TargetStateLast(v)
            | Expression::CasterStateLast(v) => {
                s.push(
                    v.cstr()
                        .wrap(("(".cstr(), ")".cstr()))
                        .color(VISIBLE_LIGHT)
                        .take(),
                );
            }
            Expression::Context(v) => s = (*v).cstr_cs(VISIBLE_BRIGHT, CstrStyle::Bold),
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
            | Expression::ToI(v)
            | Expression::UnitVec(v)
            | Expression::Sin(v)
            | Expression::Cos(v)
            | Expression::Abs(v)
            | Expression::Even(v)
            | Expression::Dbg(v)
            | Expression::Ctx(v)
            | Expression::RandomUnit(v)
            | Expression::ListCount(v)
            | Expression::RandomF(v)
            | Expression::Floor(v)
            | Expression::Fract(v)
            | Expression::Ceil(v)
            | Expression::SlotUnit(v) => {
                s.push(v.cstr().wrap(("(".cstr(), ")".cstr())).take());
            }
            Expression::Vec2EE(a, b)
            | Expression::Sum(a, b)
            | Expression::Sub(a, b)
            | Expression::Mul(a, b)
            | Expression::Div(a, b)
            | Expression::Min(a, b)
            | Expression::Max(a, b)
            | Expression::Mod(a, b)
            | Expression::And(a, b)
            | Expression::Or(a, b)
            | Expression::MaxUnit(a, b)
            | Expression::RandomUnitSubset(a, b)
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
            Expression::FilterStatusUnits(status, u) => {
                s.push(
                    status
                        .cstr_c(name_color(status))
                        .push(", ".cstr())
                        .push(u.cstr())
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
            | Expression::Status
            | Expression::AllAllyUnits
            | Expression::AllEnemyUnits
            | Expression::AllUnits
            | Expression::AllOtherUnits
            | Expression::AdjacentUnits => {}
        }
        s
    }
}
