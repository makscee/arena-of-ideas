use super::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Expression {
    Float(f32),
    Bool(bool),
    String(String),
    Vec2(f32, f32),
    Vec2EE(Box<Expression>, Box<Expression>),
    Vec2E(Box<Expression>),

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
            Expression::Float(value) => Ok(*value),
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
                let mut result = None;
                let mut entity = owner;

                loop {
                    if let Some(state) = world.get::<VarState>(entity) {
                        if let Ok(value) = state.get_value(*var, t) {
                            result = Some(value);
                            break;
                        }
                    }
                    if result.is_none() {
                        if let Some(parent) = world.get::<Parent>(entity) {
                            entity = parent.get();
                            continue;
                        }
                    }
                    break;
                }
                if let Some(result) = result {
                    result.get_float()
                } else {
                    Err(anyhow!("State var not found {var:?}"))
                }
            }
            _ => Err(anyhow!("Float not supported by {self:?}")),
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
            Expression::String(value) => {
                return Ok(value.into());
            }
            _ => {}
        }
        match self.get_float(owner, world) {
            Ok(value) => Ok(format!("{value:.2}")),
            Err(err) => Err(err),
        }
    }
}
