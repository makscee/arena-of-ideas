use ecolor::HexColor;

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, EnumIter, AsRefStr)]
pub enum Expression {
    #[default]
    One,
    Zero,

    OppositeFaction,
    SlotPosition,
    GT,
    Beat,
    PI,
    PI2,
    Age,
    Index,
    T,

    Owner,
    Caster,
    Target,
    Status,

    AllAllyUnits,
    AllEnemyUnits,
    AllUnits,
    AllOtherUnits,
    AdjacentUnits,

    FilterStatusUnits(String, Box<Expression>),
    FilterNoStatusUnits(String, Box<Expression>),
    StatusEntity(String, Box<Expression>),

    Value(VarValue),
    Context(VarName),
    OwnerState(VarName),
    TargetState(VarName),
    CasterState(VarName),
    StatusState(String, VarName),
    OwnerStateLast(VarName),
    TargetStateLast(VarName),
    CasterStateLast(VarName),
    StatusStateLast(String, VarName),
    AbilityContext(String, VarName),
    AbilityState(String, VarName),
    StatusCharges(String),
    HexColor(String),
    F(f32),
    I(i32),
    B(bool),
    S(String),
    V2(f32, f32),

    Dbg(Box<Expression>),
    Ctx(Box<Expression>),
    ToI(Box<Expression>),
    ToF(Box<Expression>),
    Vec2E(Box<Expression>),
    UnitVec(Box<Expression>),
    VX(Box<Expression>),
    VY(Box<Expression>),
    Sin(Box<Expression>),
    Cos(Box<Expression>),
    Sqr(Box<Expression>),
    Even(Box<Expression>),
    Abs(Box<Expression>),
    Floor(Box<Expression>),
    Ceil(Box<Expression>),
    Fract(Box<Expression>),
    SlotUnit(Box<Expression>),
    RandomF(Box<Expression>),
    RandomUnit(Box<Expression>),
    ListCount(Box<Expression>),

    MaxUnit(Box<Expression>, Box<Expression>),
    RandomUnitSubset(Box<Expression>, Box<Expression>),
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
            Expression::One => Ok(VarValue::Int(1)),
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
            Expression::StatusState(status, var) => Context::new_play(context.owner())
                .set_status(status.clone())
                .get_value(*var, world),
            Expression::OwnerStateLast(var) => Context::new(context.owner()).get_value(*var, world),
            Expression::TargetStateLast(var) => {
                Context::new(context.get_target()?).get_value(*var, world)
            }
            Expression::CasterStateLast(var) => {
                Context::new(context.get_caster()?).get_value(*var, world)
            }
            Expression::StatusStateLast(status, var) => Context::new(context.owner())
                .set_status(status.clone())
                .get_value(*var, world),
            Expression::AbilityContext(ability, var) => context.get_ability_var(ability, *var),
            Expression::AbilityState(ability, var) => {
                Ok(
                    TeamPlugin::get_ability_state(ability, context.get_faction(world)?, world)
                        .and_then(|s| s.get_value_at(*var, gt().play_head()).ok())
                        .unwrap_or_else(|| GameAssets::ability_default(&ability, *var)),
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
            Expression::HexColor(s) => Ok(VarValue::Color({
                let s = s.strip_prefix('#').unwrap_or(s);
                HexColor::from_str_without_hash(s)
                    .map(|c| c.color().to_color())
                    .unwrap_or_default()
            })),
            Expression::Sin(v) => Ok(v.get_float(context, world)?.sin().into()),
            Expression::Cos(v) => Ok(v.get_float(context, world)?.cos().into()),
            Expression::Sqr(v) => Ok({
                let x = v.get_float(context, world)?;
                (x * x).into()
            }),
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
            Expression::VX(x) => Ok(x.get_vec2(context, world)?.x.into()),
            Expression::VY(x) => Ok(x.get_vec2(context, world)?.y.into()),
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
            Expression::ToF(v) => Ok(v.get_float(context, world).unwrap_or_default().into()),
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
            Expression::FilterNoStatusUnits(status, units) => {
                let units = units.get_value(context, world)?.get_entity_list()?;
                Ok(units
                    .into_iter()
                    .filter(|u| Status::get_charges(&status, *u, world).unwrap_or_default() == 0)
                    .collect_vec()
                    .into())
            }
            Expression::StatusEntity(status, owner) => {
                Status::find_status_entity(owner.get_entity(context, world)?, status, world)
                    .map(|e| e.into())
            }
            Expression::Age => {
                Ok((gt().play_head() - VarState::get(context.owner(), world).birth()).into())
            }
            Expression::Index => Expression::Context(VarName::Index).get_value(context, world),
            Expression::T => Expression::Context(VarName::T).get_value(context, world),
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
                if let Some(mut rng) = ActionPlugin::resource(world).rng.take() {
                    let result = units.into_iter().choose_multiple(&mut rng, amount).into();
                    ActionPlugin::resource(world).rng = Some(rng);
                    Ok(result)
                } else {
                    Ok(units
                        .into_iter()
                        .choose_multiple(&mut thread_rng(), amount)
                        .into())
                }
            }
            Expression::RandomUnit(units) => {
                let units = units.get_value(context, world)?.get_entity_list()?;
                if let Some(mut rng) = ActionPlugin::resource(world).rng.take() {
                    let result = units
                        .into_iter()
                        .choose(&mut rng)
                        .map(|u| u.into())
                        .context("No units to choose from");
                    ActionPlugin::resource(world).rng = Some(rng);
                    result
                } else {
                    units
                        .into_iter()
                        .choose(&mut thread_rng())
                        .map(|u| u.into())
                        .context("No units to choose from")
                }
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
        match self {
            Expression::One
            | Expression::Zero
            | Expression::OppositeFaction
            | Expression::SlotPosition
            | Expression::GT
            | Expression::Beat
            | Expression::PI
            | Expression::PI2
            | Expression::Age
            | Expression::Index
            | Expression::T
            | Expression::Owner
            | Expression::Caster
            | Expression::Target
            | Expression::Status
            | Expression::AllAllyUnits
            | Expression::AllEnemyUnits
            | Expression::AllUnits
            | Expression::AllOtherUnits
            | Expression::AdjacentUnits => self.as_ref().cstr_c(VISIBLE_LIGHT),
            Expression::FilterStatusUnits(_, _)
            | Expression::FilterNoStatusUnits(_, _)
            | Expression::StatusEntity(_, _) => self.as_ref().cstr_c(PURPLE),
            Expression::Value(_)
            | Expression::Context(_)
            | Expression::OwnerState(_)
            | Expression::TargetState(_)
            | Expression::CasterState(_)
            | Expression::StatusState(_, _)
            | Expression::OwnerStateLast(_)
            | Expression::TargetStateLast(_)
            | Expression::CasterStateLast(_)
            | Expression::StatusStateLast(_, _)
            | Expression::AbilityContext(_, _)
            | Expression::AbilityState(_, _)
            | Expression::StatusCharges(_)
            | Expression::HexColor(_)
            | Expression::F(_)
            | Expression::I(_)
            | Expression::B(_)
            | Expression::S(_)
            | Expression::V2(_, _) => self.as_ref().cstr_c(CYAN),

            Expression::Dbg(_)
            | Expression::Ctx(_)
            | Expression::ToI(_)
            | Expression::ToF(_)
            | Expression::Vec2E(_)
            | Expression::UnitVec(_)
            | Expression::VX(_)
            | Expression::VY(_)
            | Expression::Sin(_)
            | Expression::Cos(_)
            | Expression::Sqr(_)
            | Expression::Even(_)
            | Expression::Abs(_)
            | Expression::Floor(_)
            | Expression::Ceil(_)
            | Expression::Fract(_)
            | Expression::SlotUnit(_)
            | Expression::RandomF(_)
            | Expression::RandomUnit(_)
            | Expression::ListCount(_) => self.as_ref().cstr_c(LIGHT_PURPLE),

            Expression::MaxUnit(_, _)
            | Expression::RandomUnitSubset(_, _)
            | Expression::Vec2EE(_, _)
            | Expression::Sum(_, _)
            | Expression::Sub(_, _)
            | Expression::Mul(_, _)
            | Expression::Div(_, _)
            | Expression::Max(_, _)
            | Expression::Min(_, _)
            | Expression::Mod(_, _)
            | Expression::And(_, _)
            | Expression::Or(_, _)
            | Expression::Equals(_, _)
            | Expression::GreaterThen(_, _)
            | Expression::LessThen(_, _)
            | Expression::If(_, _, _)
            | Expression::WithVar(_, _, _) => self.as_ref().cstr_c(PURPLE),
        }
    }
    fn cstr_expanded(&self) -> Cstr {
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
            Expression::AbilityContext(name, v)
            | Expression::AbilityState(name, v)
            | Expression::StatusState(name, v)
            | Expression::StatusStateLast(name, v) => {
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
            | Expression::ToF(v)
            | Expression::UnitVec(v)
            | Expression::VX(v)
            | Expression::VY(v)
            | Expression::Sin(v)
            | Expression::Cos(v)
            | Expression::Sqr(v)
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
                s.push(v.cstr_expanded().wrap(("(".cstr(), ")".cstr())).take());
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
                    a.cstr_expanded()
                        .push(", ".cstr())
                        .push(b.cstr_expanded())
                        .wrap(("(".cstr(), ")".cstr()))
                        .take(),
                );
            }
            Expression::If(i, t, e) => {
                s.push(
                    format!("if ")
                        .cstr()
                        .push(i.cstr_expanded().wrap(("(".cstr(), ")".cstr())).take())
                        .take(),
                )
                .push(
                    format!("then ")
                        .cstr()
                        .push(t.cstr_expanded().wrap(("(".cstr(), ")".cstr())).take())
                        .take(),
                )
                .push(
                    format!("else ")
                        .cstr()
                        .push(e.cstr_expanded().wrap(("(".cstr(), ")".cstr())).take())
                        .take(),
                );
            }
            Expression::WithVar(var, val, e) => {
                s.push(
                    var.cstr()
                        .push(val.cstr_expanded())
                        .push(e.cstr_expanded())
                        .join(&", ".cstr())
                        .wrap(("(".cstr(), ")".cstr()))
                        .take(),
                );
            }
            Expression::FilterStatusUnits(status, u)
            | Expression::FilterNoStatusUnits(status, u)
            | Expression::StatusEntity(status, u) => {
                s.push(
                    status
                        .cstr_c(name_color(status))
                        .push(", ".cstr())
                        .push(u.cstr_expanded())
                        .wrap(("(".cstr(), ")".cstr()))
                        .take(),
                );
            }

            Expression::One
            | Expression::Zero
            | Expression::OppositeFaction
            | Expression::SlotPosition
            | Expression::GT
            | Expression::Beat
            | Expression::PI
            | Expression::PI2
            | Expression::Age
            | Expression::Index
            | Expression::T
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

impl ShowEditor for Expression {
    fn wrapper() -> Option<Self> {
        Some(Self::Abs(default()))
    }
    fn show_children(&mut self, context: &Context, world: &mut World, ui: &mut Ui) {
        ui.push_id(self.as_ref().to_string(), |ui| match self {
            Expression::One
            | Expression::Zero
            | Expression::OppositeFaction
            | Expression::SlotPosition
            | Expression::GT
            | Expression::Beat
            | Expression::PI
            | Expression::PI2
            | Expression::Age
            | Expression::Index
            | Expression::T
            | Expression::Owner
            | Expression::Caster
            | Expression::Target
            | Expression::Status
            | Expression::AllAllyUnits
            | Expression::AllEnemyUnits
            | Expression::AllUnits
            | Expression::AllOtherUnits
            | Expression::AdjacentUnits
            | Expression::Value(..)
            | Expression::Context(..)
            | Expression::OwnerState(..)
            | Expression::TargetState(..)
            | Expression::CasterState(..)
            | Expression::OwnerStateLast(..)
            | Expression::TargetStateLast(..)
            | Expression::CasterStateLast(..)
            | Expression::StatusState(_, _)
            | Expression::StatusStateLast(_, _)
            | Expression::AbilityContext(_, _)
            | Expression::AbilityState(_, _)
            | Expression::StatusCharges(_)
            | Expression::HexColor(_)
            | Expression::F(_)
            | Expression::I(_)
            | Expression::B(_)
            | Expression::S(_)
            | Expression::V2(_, _) => {}
            Expression::FilterStatusUnits(_, e)
            | Expression::FilterNoStatusUnits(_, e)
            | Expression::StatusEntity(_, e)
            | Expression::Dbg(e)
            | Expression::Ctx(e)
            | Expression::ToI(e)
            | Expression::ToF(e)
            | Expression::Vec2E(e)
            | Expression::UnitVec(e)
            | Expression::VX(e)
            | Expression::VY(e)
            | Expression::Sin(e)
            | Expression::Cos(e)
            | Expression::Sqr(e)
            | Expression::Even(e)
            | Expression::Abs(e)
            | Expression::Floor(e)
            | Expression::Ceil(e)
            | Expression::Fract(e)
            | Expression::SlotUnit(e)
            | Expression::RandomF(e)
            | Expression::RandomUnit(e)
            | Expression::ListCount(e) => e.show_node("", context, world, ui),

            Expression::Sum(a, b)
            | Expression::Sub(a, b)
            | Expression::Mul(a, b)
            | Expression::Div(a, b)
            | Expression::Max(a, b)
            | Expression::Min(a, b)
            | Expression::Mod(a, b)
            | Expression::And(a, b)
            | Expression::Or(a, b)
            | Expression::Equals(a, b)
            | Expression::GreaterThen(a, b)
            | Expression::LessThen(a, b) => {
                a.show_node("a", context, world, ui);
                b.show_node("b", context, world, ui);
            }

            Expression::MaxUnit(value, units) => {
                value.show_node("value", context, world, ui);
                units.show_node("units", context, world, ui);
            }
            Expression::RandomUnitSubset(amount, units) => {
                amount.show_node("amount", context, world, ui);
                units.show_node("units", context, world, ui);
            }
            Expression::Vec2EE(x, y) => {
                x.show_node("x", context, world, ui);
                y.show_node("y", context, world, ui);
            }
            Expression::WithVar(_, value, expression) => {
                value.show_node("value", context, world, ui);
                expression.show_node("e", context, world, ui);
            }
            Expression::If(cond, th, el) => {
                cond.show_node("condition", context, world, ui);
                th.show_node("then", context, world, ui);
                el.show_node("else", context, world, ui);
            }
        });
    }

    fn show_content(&mut self, context: &Context, world: &mut World, ui: &mut Ui) {
        let value = self.get_value(context, world);
        match self {
            Expression::FilterStatusUnits(status, _)
            | Expression::FilterNoStatusUnits(status, _)
            | Expression::StatusEntity(status, _) => {
                status_selector(status, ui);
            }
            Expression::Value(v) => {
                v.cstr().label(ui);
            }
            Expression::Context(var)
            | Expression::OwnerState(var)
            | Expression::TargetState(var)
            | Expression::CasterState(var)
            | Expression::OwnerStateLast(var)
            | Expression::TargetStateLast(var)
            | Expression::CasterStateLast(var)
            | Expression::WithVar(var, ..) => {
                var_selector(var, ui);
            }
            Expression::StatusState(status, var) | Expression::StatusStateLast(status, var) => {
                status_selector(status, ui);
                var_selector(var, ui);
            }
            Expression::AbilityContext(ability, var) | Expression::AbilityState(ability, var) => {
                ability_selector(ability, ui);
                var_selector(var, ui);
            }
            Expression::StatusCharges(status) => {
                status_selector(status, ui);
            }
            Expression::HexColor(color) => {
                if let Ok(value) = value.as_ref() {
                    if let Ok(mut c32) = value.get_color32() {
                        if ui.color_edit_button_srgba(&mut c32).changed() {
                            *color = c32.to_hex();
                        }
                    }
                }
            }
            Expression::F(v) => {
                DragValue::new(v).ui(ui);
            }
            Expression::I(v) => {
                DragValue::new(v).ui(ui);
            }
            Expression::B(v) => {
                Checkbox::new(v, "").ui(ui);
            }
            Expression::S(v) => {
                Input::new("").ui_string(v, ui);
            }
            Expression::V2(x, y) => {
                DragValue::new(x).ui(ui);
                DragValue::new(y).ui(ui);
            }

            Expression::One
            | Expression::Zero
            | Expression::OppositeFaction
            | Expression::SlotPosition
            | Expression::GT
            | Expression::Beat
            | Expression::PI
            | Expression::PI2
            | Expression::Age
            | Expression::Index
            | Expression::T
            | Expression::Owner
            | Expression::Caster
            | Expression::Target
            | Expression::Status
            | Expression::AllAllyUnits
            | Expression::AllEnemyUnits
            | Expression::AllUnits
            | Expression::AllOtherUnits
            | Expression::AdjacentUnits
            | Expression::Dbg(..)
            | Expression::Ctx(..)
            | Expression::ToI(..)
            | Expression::ToF(..)
            | Expression::Vec2E(..)
            | Expression::UnitVec(..)
            | Expression::VX(..)
            | Expression::VY(..)
            | Expression::Sin(..)
            | Expression::Cos(..)
            | Expression::Sqr(..)
            | Expression::Even(..)
            | Expression::Abs(..)
            | Expression::Floor(..)
            | Expression::Ceil(..)
            | Expression::Fract(..)
            | Expression::SlotUnit(..)
            | Expression::RandomF(..)
            | Expression::RandomUnit(..)
            | Expression::ListCount(..)
            | Expression::MaxUnit(..)
            | Expression::RandomUnitSubset(..)
            | Expression::Vec2EE(..)
            | Expression::Sum(..)
            | Expression::Sub(..)
            | Expression::Mul(..)
            | Expression::Div(..)
            | Expression::Max(..)
            | Expression::Min(..)
            | Expression::Mod(..)
            | Expression::And(..)
            | Expression::Or(..)
            | Expression::Equals(..)
            | Expression::GreaterThen(..)
            | Expression::LessThen(..)
            | Expression::If(..) => {}
        };
        show_value(value, ui);
    }
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        match self {
            Expression::FilterStatusUnits(_, e)
            | Expression::FilterNoStatusUnits(_, e)
            | Expression::StatusEntity(_, e)
            | Expression::Dbg(e)
            | Expression::Ctx(e)
            | Expression::ToI(e)
            | Expression::ToF(e)
            | Expression::Vec2E(e)
            | Expression::UnitVec(e)
            | Expression::VX(e)
            | Expression::VY(e)
            | Expression::Sin(e)
            | Expression::Cos(e)
            | Expression::Sqr(e)
            | Expression::Even(e)
            | Expression::Abs(e)
            | Expression::Floor(e)
            | Expression::Ceil(e)
            | Expression::Fract(e)
            | Expression::SlotUnit(e)
            | Expression::RandomF(e)
            | Expression::RandomUnit(e)
            | Expression::ListCount(e) => [e].into(),
            Expression::MaxUnit(a, b)
            | Expression::RandomUnitSubset(a, b)
            | Expression::Vec2EE(a, b)
            | Expression::Sum(a, b)
            | Expression::Sub(a, b)
            | Expression::Mul(a, b)
            | Expression::Div(a, b)
            | Expression::Max(a, b)
            | Expression::Min(a, b)
            | Expression::Mod(a, b)
            | Expression::And(a, b)
            | Expression::Or(a, b)
            | Expression::Equals(a, b)
            | Expression::GreaterThen(a, b)
            | Expression::WithVar(_, a, b)
            | Expression::LessThen(a, b) => [a, b].into(),
            Expression::If(a, b, c) => [a, b, c].into(),
            Expression::One
            | Expression::Zero
            | Expression::OppositeFaction
            | Expression::SlotPosition
            | Expression::GT
            | Expression::Beat
            | Expression::PI
            | Expression::PI2
            | Expression::Age
            | Expression::Index
            | Expression::T
            | Expression::Owner
            | Expression::Caster
            | Expression::Target
            | Expression::Status
            | Expression::AllAllyUnits
            | Expression::AllEnemyUnits
            | Expression::AllUnits
            | Expression::AllOtherUnits
            | Expression::AdjacentUnits
            | Expression::Value(_)
            | Expression::Context(_)
            | Expression::OwnerState(_)
            | Expression::TargetState(_)
            | Expression::CasterState(_)
            | Expression::StatusState(_, _)
            | Expression::OwnerStateLast(_)
            | Expression::TargetStateLast(_)
            | Expression::CasterStateLast(_)
            | Expression::StatusStateLast(_, _)
            | Expression::AbilityContext(_, _)
            | Expression::AbilityState(_, _)
            | Expression::StatusCharges(_)
            | Expression::HexColor(_)
            | Expression::F(_)
            | Expression::I(_)
            | Expression::B(_)
            | Expression::S(_)
            | Expression::V2(_, _) => default(),
        }
    }

    fn get_variants() -> impl Iterator<Item = Self> {
        Self::iter()
    }
}

fn show_value(value: Result<VarValue>, ui: &mut Ui) {
    let w = ui.available_width();
    ui.set_max_width(ui.min_size().x);
    match value {
        Ok(v) => v
            .cstr()
            .style(CstrStyle::Small)
            .as_label(ui)
            .truncate()
            .ui(ui),
        Err(e) => e
            .to_string()
            .cstr_cs(RED, CstrStyle::Small)
            .as_label(ui)
            .truncate()
            .ui(ui),
    };
    ui.set_max_width(w);
}
