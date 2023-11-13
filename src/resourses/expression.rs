use rand::{seq::IteratorRandom, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

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
    Faction(Faction),

    StringInt(Box<Expression>),
    StringFloat(Box<Expression>),
    StringVec(Box<Expression>),

    Sin(Box<Expression>),
    Cos(Box<Expression>),
    UnitVec(Box<Expression>),
    Even(Box<Expression>),
    GameTime,
    RandomFloat,

    Sum(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),

    Owner,
    Caster,
    Target,
    SlotUnit(Box<Expression>),
    RandomUnit,

    State(VarName),
    StateLast(VarName),
    Context(VarName),
    SlotPosition,

    OwnerFaction,
    OppositeFaction,
    FactionCount(Box<Expression>),
    StatusCharges(Box<Expression>),

    Equals(Vec<Box<Expression>>),

    Hex(String),

    Beat,

    If(Box<Expression>, Box<Expression>, Box<Expression>),
}
impl Expression {
    pub fn get_value(&self, context: &Context, world: &mut World) -> Result<VarValue> {
        match self {
            Expression::RandomFloat => {
                let mut rng = ChaCha8Rng::seed_from_u64(context.owner().to_bits());
                Ok(VarValue::Float(rng.gen_range(0.0..1.0)))
            }
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
            Expression::Cos(x) => Ok(VarValue::Float(x.get_float(context, world)?.cos())),
            Expression::UnitVec(x) => {
                let x = x.get_float(context, world)?;
                let x = vec2(x.cos(), x.sin());
                Ok(VarValue::Vec2(x))
            }
            Expression::Even(x) => {
                let x = x.get_int(context, world)?;
                Ok(VarValue::Bool(x % 2 == 0))
            }
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
                let t = get_t(world);
                VarState::find_value(context.owner(), *var, t, world)
            }
            Expression::StateLast(var) => {
                VarState::get(context.owner(), world).get_value_last(*var)
            }
            Expression::Context(var) => context
                .get_var(*var, world)
                .with_context(|| format!("Var {var} was not found")),
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
            Expression::SlotUnit(index) => Ok(VarValue::Entity(
                UnitPlugin::find_unit(
                    context
                        .get_var(VarName::Faction, world)
                        .unwrap()
                        .get_faction()?,
                    index.get_int(context, world)? as usize,
                    world,
                )
                .context("No unit in slot")?,
            )),
            Expression::RandomUnit => Ok(VarValue::Entity(
                UnitPlugin::collect_faction(
                    context
                        .get_var(VarName::Faction, world)
                        .unwrap()
                        .get_faction()?,
                    world,
                )
                .into_iter()
                .filter(|x| !x.eq(&context.owner()))
                .choose(&mut thread_rng())
                .context("No other units found")?,
            )),
            Expression::OwnerFaction => Ok(VarValue::Faction(UnitPlugin::get_faction(
                context.owner(),
                world,
            ))),
            Expression::OppositeFaction => Ok(VarValue::Faction(
                UnitPlugin::get_faction(context.owner(), world).opposite(),
            )),
            Expression::Faction(faction) => Ok(VarValue::Faction(*faction)),
            Expression::FactionCount(faction) => Ok(VarValue::Int(
                UnitPlugin::collect_faction(faction.get_faction(context, world)?, world).len()
                    as i32,
            )),
            Expression::Equals(values) => {
                let mut var_values: Vec<VarValue> = default();
                for value in values {
                    let value = value.get_value(context, world)?;
                    var_values.push(value);
                }
                Ok(VarValue::Bool(var_values.into_iter().all_equal()))
            }
            Expression::Hex(color) => Ok(VarValue::Color(Color::hex(color)?)),
            Expression::StatusCharges(name) => {
                let status_name = name.get_string(context, world)?;
                for status in Status::collect_entity_statuses(context.owner(), world) {
                    let state = VarState::get(status, world);
                    if let Ok(name) = state.get_string(VarName::Name) {
                        if name.eq(&status_name) {
                            return Ok(VarValue::Int(state.get_int(VarName::Charges)?));
                        }
                    }
                }
                return Err(anyhow!("Can't find status"));
            }
            Expression::Beat => {
                const BPM: usize = 100;
                let ts = match AudioPlugin::background_position(world) {
                    Some(data) => data as f32,
                    None => GameTimer::get(world).get_t(),
                };
                let beat = (ts * BPM as f32 / 60.0) as usize;

                let t = ts * BPM as f32 / 60.0;
                let t = t - t.floor();
                let start = match beat % 2 == 0 {
                    true => -1.0,
                    false => 1.0,
                };
                let start = VarValue::Float(start);
                let result =
                    Tween::QuartOut.f(&start, &VarValue::Float(0.0), t, BPM as f32 / 60.0 * 0.5);
                return result;
            }
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
