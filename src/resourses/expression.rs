use std::f32::consts::PI;

use bevy_egui::egui::{ComboBox, DragValue};
use hex::encode;
use rand::{seq::IteratorRandom, Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use super::*;

#[derive(Serialize, Deserialize, Debug, Clone, Default, Display, PartialEq, EnumIter)]
pub enum Expression {
    #[default]
    Zero,
    GameTime,
    RandomFloat,
    PI,
    Owner,
    Caster,
    Target,
    RandomUnit,
    RandomAdjacentUnit,
    Age,
    SlotPosition,
    OwnerFaction,
    OppositeFaction,
    Beat,

    Float(f32),
    Int(i32),
    Bool(bool),
    String(String),
    Hex(String),
    Faction(Faction),
    State(VarName),
    TargetState(VarName),
    StateLast(VarName),
    Context(VarName),

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
}
impl Expression {
    pub fn get_value(&self, context: &Context, world: &mut World) -> Result<VarValue> {
        match self {
            Expression::Zero => Ok(VarValue::Int(0)),
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
            Expression::IntFloat(x) => Ok(VarValue::Float(x.get_int(context, world)? as f32)),
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
            Expression::GameTime => Ok(VarValue::Float(GameTimer::get(world).play_head())),
            Expression::PI => Ok(VarValue::Float(PI)),
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
                let t = get_play_head(world);
                VarState::find_value(context.owner(), *var, t, world)
            }
            Expression::TargetState(var) => {
                let t = get_play_head(world);
                VarState::find_value(context.target(), *var, t, world)
            }
            Expression::StateLast(var) => {
                VarState::get(context.owner(), world).get_value_last(*var)
            }
            Expression::Age => Ok(VarValue::Float(
                get_play_head(world) - VarState::find(context.owner(), world).birth,
            )),
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
            Expression::RandomAdjacentUnit => {
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
                Ok(VarValue::Entity(
                    UnitPlugin::find_unit(faction, (own_slot - min_distance) as usize, world)
                        .into_iter()
                        .chain(
                            UnitPlugin::find_unit(
                                faction,
                                (own_slot + min_distance) as usize,
                                world,
                            )
                            .into_iter(),
                        )
                        .choose(&mut thread_rng())
                        .context("No adjacent units found")?,
                ))
            }
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
                for status in Status::collect_entity_statuses(context.owner(), world) {
                    let state = VarState::get(status, world);
                    if let Ok(name) = state.get_string(VarName::Name) {
                        if name.eq(&status_name) {
                            return Ok(VarValue::Int(state.get_int(VarName::Charges)?));
                        }
                    }
                }
                Err(anyhow!("Can't find status"))
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
            Expression::GreaterThen(a, b) => Ok(VarValue::Bool(
                match VarValue::compare(
                    &a.get_value(context, world)?,
                    &b.get_value(context, world)?,
                )? {
                    std::cmp::Ordering::Greater => true,
                    _ => false,
                },
            )),
            Expression::LessThen(a, b) => Ok(VarValue::Bool(
                match VarValue::compare(
                    &a.get_value(context, world)?,
                    &b.get_value(context, world)?,
                )? {
                    std::cmp::Ordering::Less => true,
                    _ => false,
                },
            )),
            Expression::Min(a, b) => Ok(VarValue::min(
                &a.get_value(context, world)?,
                &b.get_value(context, world)?,
            )?),
            Expression::Max(a, b) => Ok(VarValue::max(
                &a.get_value(context, world)?,
                &b.get_value(context, world)?,
            )?),
            Expression::Abs(x) => x.get_value(context, world)?.abs(),
        }
    }

    pub fn get_inner(&mut self) -> Vec<&mut Box<Expression>> {
        match self {
            Expression::Zero
            | Expression::GameTime
            | Expression::RandomFloat
            | Expression::PI
            | Expression::Owner
            | Expression::Caster
            | Expression::Target
            | Expression::RandomUnit
            | Expression::RandomAdjacentUnit
            | Expression::Age
            | Expression::SlotPosition
            | Expression::OwnerFaction
            | Expression::OppositeFaction
            | Expression::Beat
            | Expression::Float(..)
            | Expression::Int(..)
            | Expression::Bool(..)
            | Expression::String(..)
            | Expression::Hex(..)
            | Expression::Faction(..)
            | Expression::State(..)
            | Expression::TargetState(..)
            | Expression::StateLast(..)
            | Expression::Context(..)
            | Expression::Vec2(..) => default(),
            Expression::StringInt(x)
            | Expression::StringFloat(x)
            | Expression::StringVec(x)
            | Expression::IntFloat(x)
            | Expression::Sin(x)
            | Expression::Cos(x)
            | Expression::Sign(x)
            | Expression::Fract(x)
            | Expression::Floor(x)
            | Expression::UnitVec(x)
            | Expression::Even(x)
            | Expression::Abs(x)
            | Expression::SlotUnit(x)
            | Expression::FactionCount(x)
            | Expression::StatusCharges(x)
            | Expression::Vec2E(x) => vec![x],

            Expression::Vec2EE(a, b)
            | Expression::Sum(a, b)
            | Expression::Sub(a, b)
            | Expression::Mul(a, b)
            | Expression::Div(a, b)
            | Expression::GreaterThen(a, b)
            | Expression::LessThen(a, b)
            | Expression::Min(a, b)
            | Expression::Equals(a, b)
            | Expression::And(a, b)
            | Expression::Or(a, b)
            | Expression::Max(a, b) => vec![a, b],
            Expression::If(a, b, c) => vec![a, b, c],
        }
    }

    pub fn set_inner(mut self, mut other: Expression) -> Self {
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

    pub fn show_editor_root(
        &mut self,
        entity: Option<Entity>,
        editing_data: &mut EditingData,
        name: String,
        show_name: bool,
        ui: &mut Ui,
        world: &mut World,
    ) {
        ui.horizontal(|ui| {
            if show_name {
                ui.label(name.clone());
            }
            self.show_editor(editing_data, name, ui);
            if let Some(entity) = entity {
                let text = match self.get_value(&Context::from_owner(entity, world), world) {
                    Ok(value) => RichText::new(format!("{value:?}")).color(hex_color!("#00ACC1")),
                    Err(err) => RichText::new(err.to_string()).color(hex_color!("#F44336")),
                };
                ui.label(text);
            }
        });
    }

    pub fn show_editor(&mut self, editing_data: &mut EditingData, name: String, ui: &mut Ui) {
        let hovered = if let Some(hovered) = editing_data.hovered.as_ref() {
            hovered.eq(&name)
        } else {
            false
        };
        let color = match hovered {
            true => hex_color!("#FF9100"),
            false => self.editor_color(),
        };
        ui.style_mut().visuals.hyperlink_color = color;
        let mut now_hovered = false;
        ui.horizontal(|ui| {
            let minus = ui.link("-");
            if minus.clicked() {
                let first: Expression = if let Some(first) = self.get_inner().first() {
                    first.as_ref().clone()
                } else {
                    default()
                };
                *self = first;
            }
            let left = ui.link(RichText::new("("));
            if left.clicked() {
                let abs = Expression::Abs(Box::new(self.clone()));
                *self = abs;
            }
            now_hovered |= left.hovered() || minus.hovered();
            ui.vertical(|ui| {
                let link = ui.link(RichText::new(format!("{self}")));
                if link.clicked() {
                    editing_data.lookup.clear();
                    link.request_focus();
                }
                now_hovered |= link.hovered();
                if link.has_focus() || link.lost_focus() {
                    let mut need_clear = false;
                    ui.horizontal_wrapped(|ui| {
                        ui.label(editing_data.lookup.to_owned());
                        Expression::iter()
                            .filter_map(|e| {
                                match e
                                    .to_string()
                                    .to_lowercase()
                                    .starts_with(editing_data.lookup.to_lowercase().as_str())
                                {
                                    true => Some(e),
                                    false => None,
                                }
                            })
                            .for_each(|e| {
                                let button = ui.button(e.to_string());
                                if button.gained_focus() || button.clicked() {
                                    *self = e.set_inner(self.clone());
                                    need_clear = true;
                                }
                            })
                    });
                    if need_clear {
                        editing_data.lookup.clear();
                    }
                }
            });

            match self {
                Expression::Zero
                | Expression::GameTime
                | Expression::RandomFloat
                | Expression::PI
                | Expression::Owner
                | Expression::Caster
                | Expression::Target
                | Expression::RandomUnit
                | Expression::RandomAdjacentUnit
                | Expression::Age
                | Expression::SlotPosition
                | Expression::OwnerFaction
                | Expression::OppositeFaction
                | Expression::Beat => {}
                Expression::Float(x) => {
                    ui.add(DragValue::new(x).speed(0.1));
                }
                Expression::Int(x) => {
                    ui.add(DragValue::new(x));
                }
                Expression::Bool(x) => {
                    ui.checkbox(x, "");
                }
                Expression::String(x) => {
                    ui.text_edit_singleline(x);
                }
                Expression::Hex(x) => {
                    let c = Color::hex(&x).unwrap_or_default().as_rgba_u8();
                    let mut c = Color32::from_rgb(c[0], c[1], c[2]);
                    if ui.color_edit_button_srgba(&mut c).changed() {
                        *x = encode(c.to_array());
                    }
                }
                Expression::Faction(x) => {
                    ComboBox::from_id_source(&name)
                        .selected_text(x.to_string())
                        .show_ui(ui, |ui| {
                            for option in Faction::iter() {
                                let text = option.to_string();
                                ui.selectable_value(x, option, text).changed();
                            }
                        });
                }
                Expression::State(x) | Expression::TargetState(x) | Expression::StateLast(x) => {
                    ComboBox::from_id_source(&name)
                        .selected_text(x.to_string())
                        .show_ui(ui, |ui| {
                            for option in VarName::iter() {
                                let text = option.to_string();
                                ui.selectable_value(x, option, text).changed();
                            }
                        });
                }
                Expression::Context(x) => {
                    ui.label(x.to_string());
                }
                Expression::Vec2(x, y) => {
                    ui.add(DragValue::new(x).speed(0.1));
                    ui.add(DragValue::new(y).speed(0.1));
                }

                Expression::Vec2E(x)
                | Expression::StringInt(x)
                | Expression::StringFloat(x)
                | Expression::StringVec(x)
                | Expression::IntFloat(x)
                | Expression::Sin(x)
                | Expression::Cos(x)
                | Expression::Sign(x)
                | Expression::Fract(x)
                | Expression::Floor(x)
                | Expression::UnitVec(x)
                | Expression::Even(x)
                | Expression::Abs(x)
                | Expression::SlotUnit(x)
                | Expression::FactionCount(x)
                | Expression::StatusCharges(x) => {
                    x.show_editor(editing_data, format!("{name}/x"), ui);
                }
                Expression::Vec2EE(a, b)
                | Expression::Sum(a, b)
                | Expression::Sub(a, b)
                | Expression::Mul(a, b)
                | Expression::Div(a, b)
                | Expression::GreaterThen(a, b)
                | Expression::LessThen(a, b)
                | Expression::Min(a, b)
                | Expression::Max(a, b)
                | Expression::Equals(a, b)
                | Expression::And(a, b)
                | Expression::Or(a, b) => {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            a.show_editor(editing_data, format!("{name}/a"), ui);
                        });
                        ui.horizontal(|ui| {
                            b.show_editor(editing_data, format!("{name}/b"), ui);
                        });
                    });
                }
                Expression::If(i, t, e) => {
                    i.show_editor(editing_data, format!("{name}/i"), ui);
                    t.show_editor(editing_data, format!("{name}/t"), ui);
                    e.show_editor(editing_data, format!("{name}/e"), ui);
                }
            }
            ui.style_mut().visuals.hyperlink_color = color;
            let right = ui.link(RichText::new(")"));
            if right.clicked() {
                for inner in self.get_inner() {
                    *inner = Box::new(Expression::Zero);
                }
            }
            now_hovered |= right.hovered();
            if now_hovered && !editing_data.hovered.as_ref().eq(&Some(&name)) {
                editing_data.hovered = Some(name.clone());
            }
        });
    }

    fn editor_color(&self) -> Color32 {
        match self {
            Expression::Zero
            | Expression::GameTime
            | Expression::RandomFloat
            | Expression::PI
            | Expression::Owner
            | Expression::Caster
            | Expression::Target
            | Expression::RandomUnit
            | Expression::RandomAdjacentUnit
            | Expression::Age
            | Expression::SlotPosition
            | Expression::OwnerFaction
            | Expression::OppositeFaction
            | Expression::Beat => hex_color!("#80D8FF"),

            Expression::Float(_)
            | Expression::Int(_)
            | Expression::Bool(_)
            | Expression::String(_)
            | Expression::Hex(_)
            | Expression::Faction(_)
            | Expression::State(_)
            | Expression::TargetState(_)
            | Expression::StateLast(_)
            | Expression::Context(_)
            | Expression::Vec2(_, _) => hex_color!("#18FFFF"),
            Expression::Vec2E(_)
            | Expression::StringInt(_)
            | Expression::StringFloat(_)
            | Expression::StringVec(_)
            | Expression::IntFloat(_)
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
            | Expression::Or(_, _) => hex_color!("#FFEB3B"),
            Expression::If(_, _, _) => hex_color!("#BA68C8"),
        }
    }
}
