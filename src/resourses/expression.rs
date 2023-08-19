use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Expression {
    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
    Vec2(f32, f32),
    Vec2EE(Box<Expression>, Box<Expression>),
    Vec2E(Box<Expression>),

    StringInt(Box<Expression>),
    StringFloat(Box<Expression>),

    Sin(Box<Expression>),
    GlobalTime,

    Sum(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),

    State(VarName),
}

impl Expression {
    pub fn get_float(&self, owner: Entity, world: &World) -> Result<f32> {
        match self {
            Expression::Float(x) => Ok(*x),
            Expression::Int(x) => Ok(*x as f32),
            Expression::Sin(x) => Ok(x.get_float(owner, world)?.sin()),
            Expression::GlobalTime => Ok(world.get_resource::<Time>().unwrap().elapsed_seconds()),
            Expression::Sum(a, b) => Ok(a.get_float(owner, world)? + b.get_float(owner, world)?),
            Expression::Sub(a, b) => Ok(a.get_float(owner, world)? - b.get_float(owner, world)?),
            Expression::Mul(a, b) => Ok(a.get_float(owner, world)? * b.get_float(owner, world)?),
            Expression::State(var) => {
                let t = world
                    .get_resource::<Time>()
                    .context("Time resource not found")?
                    .elapsed_seconds();
                VarState::find_value(owner, *var, t, world)?.get_float()
            }
            _ => Err(anyhow!("Float not supported by {self:?}")),
        }
    }

    pub fn get_int(&self, owner: Entity, world: &World) -> Result<i32> {
        match self {
            Expression::Int(value) => Ok(*value),
            Expression::State(var) => {
                let t = world
                    .get_resource::<Time>()
                    .context("Time resource not found")?
                    .elapsed_seconds();
                VarState::find_value(owner, *var, t, world)?.get_int()
            }
            _ => Err(anyhow!("Int not supported by {self:?}")),
        }
    }

    pub fn get_vec2(&self, owner: Entity, world: &World) -> Result<Vec2> {
        match self {
            Expression::Vec2EE(x, y) => {
                Ok(vec2(x.get_float(owner, world)?, y.get_float(owner, world)?))
            }
            Expression::Vec2E(x) => {
                let x = x.get_float(owner, world)?;
                Ok(vec2(x, x))
            }
            Expression::Vec2(x, y) => Ok(vec2(*x, *y)),
            Expression::Sum(a, b) => Ok(a.get_vec2(owner, world)? + b.get_vec2(owner, world)?),
            Expression::Sub(a, b) => Ok(a.get_vec2(owner, world)? - b.get_vec2(owner, world)?),
            Expression::Mul(a, b) => Ok(a.get_vec2(owner, world)? * b.get_vec2(owner, world)?),
            _ => Err(anyhow!("Vec2 not supported by {self:?}")),
        }
    }

    pub fn get_bool(&self, owner: Entity, world: &World) -> Result<bool> {
        match self {
            Expression::Bool(value) => {
                return Ok(*value);
            }
            _ => {}
        };
        match self.get_float(owner, world) {
            Ok(value) => Ok(value > 0.0),
            Err(err) => Err(err),
        }
    }

    pub fn get_string(&self, owner: Entity, world: &World) -> Result<String> {
        match self {
            Expression::String(value) => Ok(value.into()),
            Expression::StringInt(value) => Ok(value.get_int(owner, world)?.to_string()),
            Expression::StringFloat(value) => Ok(format!("{:.2}", value.get_float(owner, world)?)),
            Expression::State(var) => {
                let t = world
                    .get_resource::<Time>()
                    .context("Time resource not found")?
                    .elapsed_seconds();
                VarState::find_value(owner, *var, t, world)?.get_string()
            }
            _ => Err(anyhow!("String not supported by {self:?}")),
        }
    }
}
