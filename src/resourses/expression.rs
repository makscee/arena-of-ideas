use convert_case::{Case, Casing};
use rand::{
    seq::{IteratorRandom, SliceRandom},
    Rng, SeedableRng,
};
use rand_chacha::ChaCha8Rng;
use std::hash::{Hash, Hasher};
use std::{collections::hash_map::DefaultHasher, f32::consts::PI};

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default, Display, PartialEq, EnumIter)]
pub enum Expression {
    #[default]
    Zero,
    GameTime,
    PI,
    PI2,
    Age,
    SlotPosition,
    OwnerFaction,
    OppositeFaction,
    Beat,
    Index,

    Owner,
    Caster,
    Target,
    RandomUnit,
    RandomAdjacentUnit,
    RandomAlly,
    RandomEnemy,
    AllyUnits,
    EnemyUnits,
    AllUnits,
    AdjacentUnits,

    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
    Hex(String),
    Faction(Faction),
    State(VarName),
    StateLast(VarName),
    TargetState(VarName),
    TargetStateLast(VarName),
    Context(VarName),
    Value(VarValue),

    Vec2(f32, f32),

    Vec2E(Box<Expression>),
    StringInt(Box<Expression>),
    StringFloat(Box<Expression>),
    StringVec(Box<Expression>),
    IntFloat(Box<Expression>),
    Sin(Box<Expression>),
    Cos(Box<Expression>),
    Sign(Box<Expression>),
    Fract(Box<Expression>),
    Floor(Box<Expression>),
    UnitVec(Box<Expression>),
    Even(Box<Expression>),
    Abs(Box<Expression>),
    SlotUnit(Box<Expression>),
    FactionCount(Box<Expression>),
    StatusCharges(Box<Expression>),
    FilterMaxEnemy(Box<Expression>),
    FindUnit(Box<Expression>),
    UnitCount(Box<Expression>),
    ToInt(Box<Expression>),
    RandomFloat(Box<Expression>),

    Vec2EE(Box<Expression>, Box<Expression>),
    Sum(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    GreaterThen(Box<Expression>, Box<Expression>),
    LessThen(Box<Expression>, Box<Expression>),
    Min(Box<Expression>, Box<Expression>),
    Max(Box<Expression>, Box<Expression>),
    Equals(Box<Expression>, Box<Expression>),
    And(Box<Expression>, Box<Expression>),
    Or(Box<Expression>, Box<Expression>),

    If(Box<Expression>, Box<Expression>, Box<Expression>),

    WithVar(VarName, Box<Expression>, Box<Expression>),
}
impl Expression {
    pub fn get_value(&self, context: &Context, world: &mut World) -> Result<VarValue> {
        match self {
            Expression::Zero => Ok(VarValue::Int(0)),
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
            Expression::IntFloat(x) => Ok(VarValue::Float(x.get_int(context, world)? as f32)),
            Expression::ToInt(x) => Ok(VarValue::Int(x.get_int(context, world)?)),
            Expression::Sin(x) => Ok(VarValue::Float(x.get_float(context, world)?.sin())),
            Expression::Cos(x) => Ok(VarValue::Float(x.get_float(context, world)?.cos())),
            Expression::Sign(x) => Ok(VarValue::Float(x.get_float(context, world)?.signum())),
            Expression::Fract(x) => Ok(VarValue::Float(x.get_float(context, world)?.fract())),
            Expression::Floor(x) => Ok(VarValue::Float(x.get_float(context, world)?.floor())),
            Expression::UnitVec(x) => {
                let x = x.get_float(context, world)?;
                let x = vec2(x.cos(), x.sin());
                Ok(VarValue::Vec2(x))
            }
            Expression::Even(x) => {
                let x = x.get_int(context, world)?;
                Ok(VarValue::Bool(x % 2 == 0))
            }
            Expression::RandomFloat(x) => {
                let x = x.get_value(context, world)?;
                let mut hasher = DefaultHasher::new();
                x.hash(&mut hasher);
                let mut rng = ChaCha8Rng::seed_from_u64(hasher.finish());
                Ok(VarValue::Float(rng.gen_range(0.0..1.0)))
            }
            Expression::GameTime => Ok(VarValue::Float(GameTimer::get().play_head())),
            Expression::PI => Ok(VarValue::Float(PI)),
            Expression::PI2 => Ok(VarValue::Float(PI * 2.0)),
            Expression::Sum(a, b) => {
                VarValue::sum(&a.get_value(context, world)?, &b.get_value(context, world)?)
            }
            Expression::Sub(a, b) => {
                VarValue::sub(&a.get_value(context, world)?, &b.get_value(context, world)?)
            }
            Expression::Mul(a, b) => {
                VarValue::mul(&a.get_value(context, world)?, &b.get_value(context, world)?)
            }
            Expression::Div(a, b) => {
                VarValue::div(&a.get_value(context, world)?, &b.get_value(context, world)?)
            }
            Expression::State(var) => {
                let t = GameTimer::get().play_head();
                VarState::find_value(context.owner(), *var, t, world)
            }
            Expression::TargetState(var) => {
                let t = GameTimer::get().play_head();
                VarState::find_value(
                    context.get_target().context("No target in context")?,
                    *var,
                    t,
                    world,
                )
            }
            Expression::TargetStateLast(var) => {
                VarState::get(context.get_target().context("No target in context")?, world)
                    .get_value_last(*var)
            }
            Expression::StateLast(var) => Ok(VarState::get(context.owner(), world)
                .get_value_last(*var)
                .unwrap_or_default()),
            Expression::Age => Ok(VarValue::Float(
                GameTimer::get().play_head() - VarState::find(context.owner(), world).birth,
            )),
            Expression::Context(var) => context
                .get_var(*var, world)
                .with_context(|| format!("Var {var} was not found")),
            Expression::Index => Expression::Context(VarName::Index).get_value(context, world),
            Expression::Owner => Ok(VarValue::Entity(
                context.get_owner().context("Owner not found")?,
            )),
            Expression::Caster => Ok(VarValue::Entity(
                context.get_caster().context("Caster not found")?,
            )),
            Expression::Target => Ok(VarValue::Entity(
                context.get_target().context("Target not found")?,
            )),
            Expression::SlotPosition => Ok(VarValue::Vec2(UnitPlugin::get_entity_slot_position(
                context.owner(),
                world,
            )?)),
            Expression::AllUnits => Ok(VarValue::EntityList(
                UnitPlugin::collect_factions(
                    [
                        UnitPlugin::get_faction(context.owner(), world),
                        UnitPlugin::get_faction(context.owner(), world).opposite(),
                    ]
                    .into(),
                    world,
                )
                .into_iter()
                .map(|(u, _)| u)
                .collect_vec(),
            )),
            Expression::AdjacentUnits => {
                let own_slot = context.get_var(VarName::Slot, world).unwrap().get_int()?;
                let faction = context
                    .get_var(VarName::Faction, world)
                    .unwrap()
                    .get_faction()?;
                let mut min_distance = 999999;
                for unit in UnitPlugin::collect_faction(faction, world) {
                    let state = VarState::get(unit, world);
                    if state.get_int(VarName::Hp)? <= 0 {
                        continue;
                    }
                    let slot = state.get_int(VarName::Slot)?;
                    let delta = (slot - own_slot).abs();
                    if delta == 0 {
                        continue;
                    }
                    min_distance = min_distance.min(delta);
                }
                Ok(VarValue::EntityList(
                    UnitPlugin::find_unit(faction, (own_slot - min_distance) as usize, world)
                        .into_iter()
                        .chain(UnitPlugin::find_unit(
                            faction,
                            (own_slot + min_distance) as usize,
                            world,
                        ))
                        .collect_vec(),
                ))
            }
            Expression::RandomAdjacentUnit => Ok(VarValue::Entity(
                *Self::AdjacentUnits
                    .get_value(context, world)?
                    .get_entity_list()?
                    .choose(&mut thread_rng())
                    .context("No adjacent units found")?,
            )),
            Expression::AllyUnits => Ok(VarValue::EntityList(UnitPlugin::collect_faction(
                UnitPlugin::get_faction(context.owner(), world),
                world,
            ))),
            Expression::EnemyUnits => Ok(VarValue::EntityList(UnitPlugin::collect_faction(
                UnitPlugin::get_faction(context.owner(), world).opposite(),
                world,
            ))),
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
            Expression::RandomAlly => Self::RandomUnit.get_value(
                context.clone().set_var(
                    VarName::Faction,
                    VarValue::Faction(UnitPlugin::get_faction(context.owner(), world)),
                ),
                world,
            ),
            Expression::RandomEnemy => Self::RandomUnit.get_value(
                context.clone().set_var(
                    VarName::Faction,
                    VarValue::Faction(UnitPlugin::get_faction(context.owner(), world).opposite()),
                ),
                world,
            ),
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
            Expression::UnitCount(condition) => {
                let faction = context
                    .get_var(VarName::Faction, world)
                    .context("No faction in context")?
                    .get_faction()?;
                let mut cnt = 0;
                for unit in UnitPlugin::collect_faction(faction, world) {
                    let context = Context::from_owner(unit, world);
                    if condition.get_bool(&context, world)? {
                        cnt += 1;
                    }
                }
                Ok(VarValue::Int(cnt))
            }
            Expression::Equals(a, b) => Ok(VarValue::Bool(
                a.get_value(context, world)?
                    .eq(&b.get_value(context, world)?),
            )),
            Expression::And(a, b) => Ok(VarValue::Bool(
                a.get_bool(context, world)? && b.get_bool(context, world)?,
            )),
            Expression::Or(a, b) => Ok(VarValue::Bool(
                a.get_bool(context, world)? || b.get_bool(context, world)?,
            )),
            Expression::Hex(color) => Ok(VarValue::Color(Color::hex(color)?)),
            Expression::StatusCharges(name) => {
                let status_name = name.get_string(context, world)?;
                for status in Status::collect_unit_statuses(context.owner(), world) {
                    let state = VarState::get(status, world);
                    if let Ok(name) = state.get_string(VarName::Name) {
                        if name.eq(&status_name) {
                            return Ok(VarValue::Int(state.get_int(VarName::Charges)?));
                        }
                    }
                }
                Ok(VarValue::Int(0))
            }
            Expression::FilterMaxEnemy(value) => {
                let faction = Self::OppositeFaction.get_faction(context, world)?;
                let (unit, _) = UnitPlugin::collect_faction(faction, world)
                    .into_iter()
                    .filter_map(
                        |u| match value.get_value(&Context::from_owner(u, world), world) {
                            Ok(v) => Some((u, v)),
                            Err(_) => None,
                        },
                    )
                    .reduce(
                        |(ae, av), (be, bv)| match VarValue::compare(&av, &bv).unwrap() {
                            std::cmp::Ordering::Less => (be, bv),
                            std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => (ae, av),
                        },
                    )
                    .context("Failed to filer max enemy")?;
                Ok(VarValue::Entity(unit))
            }
            Expression::FindUnit(condition) => {
                let faction = context
                    .get_var(VarName::Faction, world)
                    .context("No Faction var in context")?
                    .get_faction()?;
                let unit = UnitPlugin::collect_faction(faction, world)
                    .into_iter()
                    .find(|u| {
                        condition
                            .get_bool(&Context::from_owner(*u, world), world)
                            .unwrap_or_default()
                    })
                    .context("Failed to find unit")?;
                Ok(VarValue::Entity(unit))
            }
            Expression::Beat => {
                let beat = AudioPlugin::beat_index(world);
                let to_next = AudioPlugin::to_next_beat(world);
                let timeframe = AudioPlugin::beat_timeframe();
                let start = match beat % 2 == 0 {
                    true => -1.0,
                    false => 1.0,
                };
                let start = VarValue::Float(start);
                let t = timeframe - to_next;

                Tween::QuartOut.f(&start, &VarValue::Float(0.0), t, timeframe * 0.5)
            }
            Expression::If(cond, th, el) => {
                if cond.get_bool(context, world)? {
                    th.get_value(context, world)
                } else {
                    el.get_value(context, world)
                }
            }
            Expression::GreaterThen(a, b) => Ok(VarValue::Bool(matches!(
                VarValue::compare(&a.get_value(context, world)?, &b.get_value(context, world)?,)?,
                std::cmp::Ordering::Greater
            ))),
            Expression::LessThen(a, b) => Ok(VarValue::Bool(matches!(
                VarValue::compare(&a.get_value(context, world)?, &b.get_value(context, world)?,)?,
                std::cmp::Ordering::Less
            ))),
            Expression::Min(a, b) => Ok(VarValue::min(
                &a.get_value(context, world)?,
                &b.get_value(context, world)?,
            )?),
            Expression::Max(a, b) => Ok(VarValue::max(
                &a.get_value(context, world)?,
                &b.get_value(context, world)?,
            )?),
            Expression::Abs(x) => x.get_value(context, world)?.abs(),
            Expression::Value(v) => Ok(v.clone()),
            Expression::WithVar(var, value, e) => e.get_value(
                context
                    .clone()
                    .set_var(*var, value.get_value(context, world)?),
                world,
            ),
        }
    }

    pub fn get_inner(&mut self) -> Vec<&mut Box<Self>> {
        match self {
            Self::Zero
            | Self::GameTime
            | Self::PI
            | Self::PI2
            | Self::Owner
            | Self::Caster
            | Self::Target
            | Self::RandomUnit
            | Self::RandomAdjacentUnit
            | Self::RandomAlly
            | Self::RandomEnemy
            | Self::Age
            | Self::SlotPosition
            | Self::OwnerFaction
            | Self::OppositeFaction
            | Self::Beat
            | Self::AllUnits
            | Self::AdjacentUnits
            | Self::AllyUnits
            | Self::EnemyUnits
            | Self::Index
            | Self::Float(..)
            | Self::Int(..)
            | Self::Bool(..)
            | Self::String(..)
            | Self::Hex(..)
            | Self::Faction(..)
            | Self::State(..)
            | Self::TargetState(..)
            | Self::StateLast(..)
            | Self::TargetStateLast(..)
            | Self::Context(..)
            | Self::Value(..)
            | Self::Vec2(..) => default(),
            Self::StringInt(x)
            | Self::StringFloat(x)
            | Self::StringVec(x)
            | Self::IntFloat(x)
            | Self::ToInt(x)
            | Self::Sin(x)
            | Self::Cos(x)
            | Self::Sign(x)
            | Self::Fract(x)
            | Self::Floor(x)
            | Self::UnitVec(x)
            | Self::Even(x)
            | Self::Abs(x)
            | Self::SlotUnit(x)
            | Self::FactionCount(x)
            | Self::StatusCharges(x)
            | Self::FilterMaxEnemy(x)
            | Self::FindUnit(x)
            | Self::UnitCount(x)
            | Self::RandomFloat(x)
            | Self::Vec2E(x) => vec![x],

            Self::Vec2EE(a, b)
            | Self::Sum(a, b)
            | Self::Sub(a, b)
            | Self::Mul(a, b)
            | Self::Div(a, b)
            | Self::GreaterThen(a, b)
            | Self::LessThen(a, b)
            | Self::Min(a, b)
            | Self::Equals(a, b)
            | Self::And(a, b)
            | Self::Or(a, b)
            | Self::Max(a, b)
            | Self::WithVar(_, a, b) => vec![a, b],
            Self::If(a, b, c) => vec![a, b, c],
        }
    }

    pub fn set_inner(mut self, mut other: Self) -> Self {
        let inner_self = self.get_inner();
        let inner_other = other.get_inner();
        for (i, e) in inner_self.into_iter().enumerate() {
            if inner_other.len() <= i {
                break;
            }
            let oe = inner_other.get(i).unwrap().as_ref().clone();
            *e = Box::new(oe);
        }
        self
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

    pub fn editor_color(&self) -> Color32 {
        match self {
            Expression::Zero
            | Expression::GameTime
            | Expression::PI
            | Expression::PI2
            | Expression::Owner
            | Expression::Caster
            | Expression::Target
            | Expression::RandomUnit
            | Expression::RandomAdjacentUnit
            | Expression::RandomAlly
            | Expression::RandomEnemy
            | Expression::Age
            | Expression::SlotPosition
            | Expression::OwnerFaction
            | Expression::AllyUnits
            | Expression::EnemyUnits
            | Expression::AllUnits
            | Expression::AdjacentUnits
            | Expression::OppositeFaction
            | Expression::Index
            | Expression::Beat => hex_color!("#80D8FF"),

            Expression::Float(_)
            | Expression::Int(_)
            | Expression::Bool(_)
            | Expression::String(_)
            | Expression::Hex(_)
            | Expression::Faction(_)
            | Expression::State(_)
            | Expression::StateLast(_)
            | Expression::TargetState(_)
            | Expression::TargetStateLast(_)
            | Expression::Context(_)
            | Expression::Value(_)
            | Expression::Vec2(_, _) => hex_color!("#18FFFF"),
            Expression::Vec2E(_)
            | Expression::StringInt(_)
            | Expression::StringFloat(_)
            | Expression::StringVec(_)
            | Expression::IntFloat(_)
            | Expression::ToInt(_)
            | Expression::Sin(_)
            | Expression::Cos(_)
            | Expression::Sign(_)
            | Expression::Fract(_)
            | Expression::Floor(_)
            | Expression::UnitVec(_)
            | Expression::Even(_)
            | Expression::Abs(_)
            | Expression::SlotUnit(_)
            | Expression::FactionCount(_)
            | Expression::FilterMaxEnemy(_)
            | Expression::FindUnit(_)
            | Expression::UnitCount(_)
            | Expression::RandomFloat(_)
            | Expression::StatusCharges(_) => hex_color!("#448AFF"),
            Expression::Vec2EE(_, _)
            | Expression::Sum(_, _)
            | Expression::Sub(_, _)
            | Expression::Mul(_, _)
            | Expression::Div(_, _)
            | Expression::GreaterThen(_, _)
            | Expression::LessThen(_, _)
            | Expression::Min(_, _)
            | Expression::Max(_, _)
            | Expression::Equals(_, _)
            | Expression::And(_, _)
            | Expression::Or(_, _)
            | Expression::WithVar(_, _, _) => hex_color!("#FFEB3B"),
            Expression::If(_, _, _) => hex_color!("#BA68C8"),
        }
    }

    pub fn get_description_string(&self) -> String {
        match self {
            Expression::Zero
            | Expression::GameTime
            | Expression::PI
            | Expression::PI2
            | Expression::Age
            | Expression::SlotPosition
            | Expression::OwnerFaction
            | Expression::OppositeFaction
            | Expression::Beat
            | Expression::Owner
            | Expression::Caster
            | Expression::Target
            | Expression::RandomUnit
            | Expression::RandomAdjacentUnit
            | Expression::RandomAlly
            | Expression::RandomEnemy
            | Expression::AllyUnits
            | Expression::EnemyUnits
            | Expression::AllUnits
            | Expression::Index
            | Expression::AdjacentUnits => self.to_string().to_case(Case::Lower),
            Expression::Float(v) => v.to_string(),
            Expression::Int(v) => v.to_string(),
            Expression::Bool(v) => v.to_string(),
            Expression::String(v) => v.to_string(),
            Expression::Hex(v) => v.to_string(),
            Expression::Faction(v) => v.to_string(),
            Expression::State(v) => format!("{self}({v})"),
            Expression::StateLast(v) => format!("{self}({v})"),
            Expression::TargetState(v) => format!("{self}({v})"),
            Expression::TargetStateLast(v) => format!("{self}({v})"),
            Expression::Context(v) => format!("{self}({v})"),
            Expression::Value(v) => format!("{self}({v})"),
            Expression::Vec2(x, y) => format!("({x}, {y})"),
            Expression::Vec2E(x) => format!("({x}, {x})"),
            Expression::StringInt(v)
            | Expression::StringFloat(v)
            | Expression::StringVec(v)
            | Expression::IntFloat(v)
            | Expression::ToInt(v)
            | Expression::Sin(v)
            | Expression::Cos(v)
            | Expression::Sign(v)
            | Expression::Fract(v)
            | Expression::Floor(v)
            | Expression::UnitVec(v)
            | Expression::Even(v)
            | Expression::Abs(v)
            | Expression::SlotUnit(v)
            | Expression::FactionCount(v)
            | Expression::FilterMaxEnemy(v)
            | Expression::FindUnit(v)
            | Expression::UnitCount(v)
            | Expression::RandomFloat(v)
            | Expression::StatusCharges(v) => format!(
                "{} ({})",
                self.to_string().to_case(Case::Lower),
                v.get_description_string().to_case(Case::Title)
            ),
            Expression::Vec2EE(x, y)
            | Expression::Sum(x, y)
            | Expression::Sub(x, y)
            | Expression::Mul(x, y)
            | Expression::Div(x, y)
            | Expression::GreaterThen(x, y)
            | Expression::LessThen(x, y)
            | Expression::Min(x, y)
            | Expression::Max(x, y)
            | Expression::Equals(x, y)
            | Expression::And(x, y)
            | Expression::Or(x, y) => format!(
                "{self}({}, {})",
                x.get_description_string().to_case(Case::Title),
                y.get_description_string().to_case(Case::Title)
            ),
            Expression::If(x, y, z) => format!(
                "{self}({}, {}, {})",
                x.get_description_string().to_case(Case::Title),
                y.get_description_string().to_case(Case::Title),
                z.get_description_string().to_case(Case::Title),
            ),
            Expression::WithVar(var, val, e) => format!(
                "{self}({}, {}, {})",
                var.to_string().to_case(Case::Title),
                val.get_description_string().to_case(Case::Title),
                e.get_description_string().to_case(Case::Title),
            ),
        }
    }
}
