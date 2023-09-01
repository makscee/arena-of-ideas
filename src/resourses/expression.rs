use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Expression {
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
    Entity(Entity),
    Vec2(f32, f32),
    Vec2EE(Box<Expression>, Box<Expression>),
    Vec2E(Box<Expression>),

    StringInt(Box<Expression>),
    StringFloat(Box<Expression>),
    StringVec(Box<Expression>),

    Sin(Box<Expression>),
    GameTime,

    Sum(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),

    Owner,
    Caster,
    Target,

    State(VarName),
    Context(VarName),
    SlotPosition,
}

impl Expression {
    pub fn get_value(&self, context: &Context, world: &World) -> Result<VarValue> {
        match self {
            Expression::Float(x) => Ok(VarValue::Float(*x)),
            Expression::Int(x) => Ok(VarValue::Int(*x)),
            Expression::Bool(x) => Ok(VarValue::Bool(*x)),
            Expression::String(x) => Ok(VarValue::String(x.into())),
            Expression::Vec2(x, y) => Ok(VarValue::Vec2(vec2(*x, *y))),
            Expression::Vec2EE(x, y) => Ok(VarValue::Vec2(vec2(
                x.get_float(context, world)?,
                y.get_float(context, world)?,
            ))),
            Expression::Vec2E(x) => {
                let x = x.get_float(context, world)?;
                Ok(VarValue::Vec2(vec2(x, x)))
            }
            Expression::StringInt(x) => {
                Ok(VarValue::String(x.get_int(context, world)?.to_string()))
            }
            Expression::StringFloat(x) => {
                Ok(VarValue::String(x.get_float(context, world)?.to_string()))
            }
            Expression::StringVec(x) => {
                let Vec2 { x, y } = x.get_vec2(context, world)?;
                Ok(VarValue::String(format!("({x:.1}:{y:.1})")))
            }
            Expression::Sin(x) => Ok(VarValue::Float(x.get_float(context, world)?.sin())),
            Expression::GameTime => Ok(VarValue::Float(GameTimer::get(world).get_t())),
            Expression::Sum(a, b) => {
                VarValue::sum(&a.get_value(context, world)?, &b.get_value(context, world)?)
            }
            Expression::Sub(a, b) => {
                VarValue::sub(&a.get_value(context, world)?, &b.get_value(context, world)?)
            }
            Expression::Mul(a, b) => {
                VarValue::mul(&a.get_value(context, world)?, &b.get_value(context, world)?)
            }
            Expression::State(var) => {
                let t = GameTimer::get(world).get_t();
                VarState::find_value(context.owner(), *var, t, world)
            }
            Expression::Context(var) => context.get_var(*var, world).context("Var not found"),
            Expression::Owner => Ok(VarValue::Entity(
                context.get_owner().context("Owner not found")?,
            )),
            Expression::Caster => Ok(VarValue::Entity(
                context.get_caster().context("Caster not found")?,
            )),
            Expression::Target => Ok(VarValue::Entity(
                context.get_target().context("Target not found")?,
            )),
            Expression::Entity(x) => Ok(VarValue::Entity(*x)),
            Expression::SlotPosition => Ok(VarValue::Vec2(UnitPlugin::get_entity_slot_position(
                context.owner(),
                world,
            )?)),
        }
    }

    pub fn get_float(&self, context: &Context, world: &World) -> Result<f32> {
        self.get_value(context, world)?.get_float()
    }

    pub fn get_int(&self, context: &Context, world: &World) -> Result<i32> {
        self.get_value(context, world)?.get_int()
    }

    pub fn get_vec2(&self, context: &Context, world: &World) -> Result<Vec2> {
        self.get_value(context, world)?.get_vec2()
    }

    pub fn get_bool(&self, context: &Context, world: &World) -> Result<bool> {
        self.get_value(context, world)?.get_bool()
    }

    pub fn get_string(&self, context: &Context, world: &World) -> Result<String> {
        self.get_value(context, world)?.get_string()
    }
}
