use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, EnumIter, AsRefStr)]
pub enum Expression {
    #[default]
    Zero,

    OppositeFaction,

    Owner,
    Caster,
    Target,
    SlotUnit(Box<Expression>),

    Value(VarValue),
    Context(VarName),
    OwnerState(VarName),
    TargetState(VarName),
    CasterState(VarName),
    StatusCharges(String),
    HexColor(String),

    Sin(Box<Expression>),
    Cos(Box<Expression>),
    FactionCount(Box<Expression>),

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
            Expression::Zero => Ok(VarValue::None),
            Expression::Value(v) => Ok(v.clone()),
            Expression::Context(var) => context.get_var(*var, world),
            Expression::OwnerState(var) => {
                VarState::find_value_at(context.owner(), *var, GameTimer::get().play_head(), world)
            }
            Expression::TargetState(var) => VarState::find_value_at(
                context.get_target()?,
                *var,
                GameTimer::get().play_head(),
                world,
            ),
            Expression::CasterState(var) => VarState::find_value_at(
                context.get_caster()?,
                *var,
                GameTimer::get().play_head(),
                world,
            ),
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
