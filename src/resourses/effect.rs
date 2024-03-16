use std::{collections::VecDeque, ops::Deref};

use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, Default, Display, PartialEq, EnumIter)]
pub enum Effect {
    #[default]
    Noop,
    Kill,
    FullCopy,
    Debug(Expression),
    Text(Expression),
    Damage(Option<Expression>),
    AoeFaction(Expression, Box<Effect>),
    WithTarget(Expression, Box<Effect>),
    WithOwner(Expression, Box<Effect>),
    List(Vec<Box<Effect>>),
    ListSpread(Vec<Box<Effect>>),
    WithVar(VarName, Expression, Box<Effect>),
    StateAddVar(VarName, Expression, Expression),
    AbilityStateAddVar(String, VarName, Expression),
    UseAbility(String, i32),
    Summon(String),
    AddStatus(String),
    Vfx(String),
    SendEvent(Event),
    RemoveLocalTrigger,
    If(Expression, Box<Effect>, Box<Effect>),
    Repeat(Expression, Box<Effect>),
}

impl Effect {
    pub fn invoke(&self, context: &mut Context, world: &mut World) -> Result<()> {
        debug!("Processing {:?}\n{}", self, context);
        match self {
            Effect::Damage(value) => {
                let target = context.get_target().context("Target not found")?;
                let owner = context.get_owner().context("Owner not found")?;
                let mut value = match value {
                    Some(value) => value.get_value(context, world)?,
                    None => context
                        .get_var(VarName::Atk, world)
                        .context("Can't find ATK")?,
                };
                debug!("Damage {} {target:?}", value.to_string());
                Event::IncomingDamage {
                    owner: target,
                    value: value.get_int()?,
                }
                .send_with_context(context.clone(), world)
                .map(&mut value, world);
                debug!("Value after map {value:?}");
                let value = value.get_int()?;
                if value > 0 {
                    let new_hp = VarState::get(target, world).get_int(VarName::Hp)? - value;
                    VarState::get_mut(target, world)
                        .push_back(VarName::Hp, VarChange::new(VarValue::Int(new_hp)));
                    VarState::get_mut(target, world).push_back(
                        VarName::LastAttacker,
                        VarChange::new(VarValue::Entity(context.owner())),
                    );
                    Event::DamageTaken {
                        owner: target,
                        value,
                    }
                    .send_with_context(context.clone(), world);
                    Event::DamageDealt {
                        owner,
                        target,
                        value,
                    }
                    .send_with_context(context.clone(), world);
                    Pools::get_vfx("pain", world)
                        .set_parent(context.target())
                        .unpack(world)?;
                }
                let value = value.max(0);
                Vfx::show_text(
                    format!("-{value}"),
                    Color::ORANGE_RED,
                    context.target(),
                    context,
                    world,
                )?;
            }
            Effect::Kill => {
                let target = context.get_target().context("Target not found")?;
                VarState::get_mut(target, world)
                    .push_back(
                        VarName::LastAttacker,
                        VarChange::new(VarValue::Entity(context.owner())),
                    )
                    .change_int(VarName::Hp, -9999999);
                Pools::get_vfx("text", world)
                    .clone()
                    .set_var(
                        VarName::Position,
                        VarValue::Vec2(UnitPlugin::get_unit_position(context.target(), world)?),
                    )
                    .set_var(VarName::Text, VarValue::String("Kill".to_string()))
                    .set_var(VarName::Color, VarValue::Color(Color::RED))
                    .unpack(world)?;
            }
            Effect::Debug(msg) => {
                let msg = msg.get_string(context, world)?;
                debug!("Debug effect: {msg}");
            }
            Effect::Noop => {}
            Effect::UseAbility(ability, mult) => {
                let effect = Pools::get_ability(ability, world)
                    .with_context(|| format!("Ability not found {ability}"))?
                    .effect
                    .clone();
                let color = Pools::get_ability_house(ability, world)
                    .with_context(|| format!("Failed to find house for ability {ability}"))?
                    .color
                    .clone()
                    .into();
                let faction = context
                    .get_var(VarName::Faction, world)
                    .context("Faction absent")?
                    .get_faction()?;
                if let Some(ability_state) = PackedTeam::get_ability_state(faction, ability, world)
                {
                    for (var, history) in ability_state.history.iter() {
                        if let Some(value) = history.get_last() {
                            context.set_ability_var(ability.to_owned(), *var, value);
                        }
                    }
                }
                Vfx::show_text(
                    format!("Use {ability}"),
                    color,
                    context.owner(),
                    context,
                    world,
                )?;
                {
                    let mut context = context
                        .clone()
                        .set_var(VarName::Color, VarValue::Color(color))
                        .take();

                    context.set_var(
                        VarName::Charges,
                        VarValue::Int(
                            context
                                .get_var(VarName::Level, world)
                                .map(|v| v.get_int().unwrap())
                                .unwrap_or(1)
                                * (*mult).max(1),
                        ),
                    );

                    ActionPlugin::action_push_front(effect, context, world);
                }
            }
            Effect::Summon(name) => {
                let unit = Pools::get_summon(name, world)
                    .with_context(|| format!("Summon unit not found {name}"))?
                    .clone();
                let color = Pools::get_summon_house(name, world)
                    .with_context(|| format!("Failed to find house for summon {name}"))?
                    .color
                    .clone()
                    .into();
                Pools::get_vfx("text", world)
                    .clone()
                    .set_var(
                        VarName::Position,
                        VarState::get(context.owner(), world).get_value_last(VarName::Position)?,
                    )
                    .set_var(VarName::Text, VarValue::String(format!("Summon {name}")))
                    .set_var(VarName::Color, VarValue::Color(color))
                    .unpack(world)?;
                {
                    let mut context = context
                        .clone()
                        .set_var(VarName::Color, VarValue::Color(color))
                        .take();
                    if context.get_var(VarName::Charges, world).is_none() {
                        context.set_var(
                            VarName::Charges,
                            context
                                .get_var(VarName::Level, world)
                                .unwrap_or(VarValue::Int(1)),
                        );
                    }
                    let faction = context
                        .get_var(VarName::Faction, world)
                        .context("No faction in context")?
                        .get_faction()?;
                    let parent =
                        PackedTeam::find_entity(faction, world).context("Team not found")?;
                    let entity = unit.unpack(parent, None, world);
                    UnitPlugin::fill_slot_gaps(faction, world);
                    UnitPlugin::translate_to_slots(world);
                    Event::Summon(entity).send_with_context(context.clone(), world);
                }
            }
            Effect::AddStatus(status) => {
                let charges = context
                    .get_var(VarName::Charges, world)
                    .unwrap_or(VarValue::Int(1))
                    .get_int()?;
                let color = Pools::get_status_house(status, world)
                    .unwrap()
                    .color
                    .clone()
                    .into();
                Status::change_charges(status, context.target(), charges, world)?;
                Pools::get_vfx("text", world)
                    .clone()
                    .set_var(
                        VarName::Position,
                        VarState::get(context.target(), world).get_value_last(VarName::Position)?,
                    )
                    .set_var(
                        VarName::Text,
                        VarValue::String(format!(
                            "{status} {}",
                            match charges.is_positive() {
                                true => format!("+{charges}"),
                                false => charges.to_string(),
                            }
                        )),
                    )
                    .set_var(VarName::Color, VarValue::Color(color))
                    .unpack(world)?;
            }
            Effect::List(list) => {
                for effect in list.into_iter().rev() {
                    ActionPlugin::action_push_front(effect.deref().clone(), context.clone(), world);
                }
            }
            Effect::ListSpread(list) => {
                for effect in list {
                    ActionPlugin::action_push_front(effect.deref().clone(), context.clone(), world);
                }
            }
            Effect::AoeFaction(faction, effect) => {
                for unit in UnitPlugin::collect_faction(faction.get_faction(context, world)?, world)
                {
                    let context = context.clone().set_target(unit, world).take();
                    ActionPlugin::action_push_front(effect.deref().clone(), context, world);
                }
            }
            Effect::Text(text) => {
                let text = text.get_string(context, world)?;
                Pools::get_vfx("text", world)
                    .clone()
                    .set_var(
                        VarName::Position,
                        VarValue::Vec2(UnitPlugin::get_unit_position(context.owner(), world)?),
                    )
                    .set_var(VarName::Text, VarValue::String(text))
                    .set_var(VarName::Color, VarValue::Color(Color::PINK))
                    .unpack(world)?;
            }
            Effect::Vfx(name) => {
                let owner_pos = UnitPlugin::get_unit_position(context.owner(), world)?;
                let delta = UnitPlugin::get_unit_position(context.target(), world)? - owner_pos;

                Pools::get_vfx(name, world)
                    .clone()
                    .attach_context(context)
                    .set_var(VarName::Delta, VarValue::Vec2(delta))
                    .set_var(VarName::Position, VarValue::Vec2(owner_pos))
                    .set_var(
                        VarName::Color,
                        context
                            .get_var(VarName::Color, world)
                            .context("Color not found in context")?,
                    )
                    .unpack(world)?;
            }
            Effect::WithTarget(target, effect) => {
                let target = target.get_value(context, world)?;
                let targets = if let Ok(mut targets) = target.get_entity_list() {
                    targets
                        .sort_by_key(|e| -VarState::get(*e, world).get_int(VarName::Slot).unwrap());
                    targets
                } else {
                    vec![target.get_entity()?]
                };
                let delay = 0.2;
                for target in targets {
                    let context = context.set_target(target, world).clone();
                    ActionPlugin::action_push_front_with_delay(
                        effect.deref().clone(),
                        context,
                        delay,
                        world,
                    );
                }
            }
            Effect::WithOwner(owner, effect) => {
                let context = context
                    .set_owner(owner.get_entity(context, world)?, world)
                    .clone();
                ActionPlugin::action_push_front(effect.deref().clone(), context, world);
            }
            Effect::WithVar(var, value, effect) => {
                let context = context
                    .set_var(*var, value.get_value(context, world)?)
                    .clone();
                ActionPlugin::action_push_front(effect.deref().clone(), context, world);
            }
            Effect::StateAddVar(var, target, value) => {
                let target = target.get_entity(context, world)?;
                let value = value.get_value(context, world)?;
                let mut state = VarState::try_get_mut(target, world)?;
                let value = match state.get_value_last(*var) {
                    Ok(prev) => VarValue::sum(&value, &prev)?,
                    Err(_) => value,
                };
                state.push_back(*var, VarChange::new(value));
            }
            Effect::AbilityStateAddVar(ability, var, value) => {
                let value = value.get_value(context, world)?;
                let faction = context
                    .get_var(VarName::Faction, world)
                    .context("Faction absent")?
                    .get_faction()?;
                let mut states = PackedTeam::get_ability_states_mut(faction, world)
                    .context("Ability states absent")?;
                let state = states.0.entry(ability.to_owned()).or_default();
                let value = match state.get_value_last(*var) {
                    Ok(prev) => VarValue::sum(&value, &prev)?,
                    Err(_) => value,
                };
                state.push_back(*var, VarChange::new(value.clone()));
                let color = Pools::get_ability_house(ability, world)
                    .with_context(|| format!("Failed to find house for ability {ability}"))?
                    .color
                    .clone()
                    .into();
                Vfx::show_text(
                    format!("{ability} {var} add {value}"),
                    color,
                    context.owner(),
                    context,
                    world,
                )?;
            }
            Effect::FullCopy => {
                let owner = context.owner();
                let target = context.target();
                let history = VarState::get(target, world).history.clone();
                for (var, history) in history.into_iter() {
                    if var.eq(&VarName::Position)
                        || var.eq(&VarName::Slot)
                        || var.eq(&VarName::Name)
                    {
                        continue;
                    }
                    if let Some(value) = history.get_last() {
                        VarState::get_mut(owner, world).push_back(var, VarChange::new(value));
                    }
                }
                if !SkipVisual::active(world) {
                    Representation::pack(target, world).unpack(owner, world);
                }
                // let source = &world.get::<Unit>(target).unwrap().source;
                // source
                //     .representation
                //     .clone()
                //     .unpack(None, Some(owner), world);
                // if let Some(entity) = PackedUnit::get_representation_entity(owner, world) {
                //     world.get_entity_mut(entity).unwrap().despawn_recursive();
                // }
                for entity in Status::collect_unit_statuses(owner, world) {
                    world.entity_mut(entity).despawn_recursive();
                }
                for entity in Status::collect_unit_statuses(target, world) {
                    let status = world.get::<Status>(entity).unwrap();
                    if let Some(status) = Pools::get_status(&status.name, world) {
                        let status = status.clone().unpack(owner, world);
                        for (var, history) in
                            VarState::get(entity, world).history.clone().into_iter()
                        {
                            if let Some(value) = history.get_last() {
                                VarState::get_mut(status, world)
                                    .push_back(var, VarChange::new(value));
                            }
                        }
                    } else {(
    name: "Enhancer",
    hp: 1,
    atk: 0,
    stacks: 1,
    level: 1,
    houses: "Mages",
    description: "%trigger → [Magic Missile] {+1} {DMG}",
    trigger: Fire(
        trigger: BattleStart,
        target: Owner,
        effect: AbilityStateAddVar("Magic Missile", Value, Int(1)),
        period: 0,
    ),
    representation: (
        material: Shape(
            shape: Circle(
                radius: Max(Sin(Sum(Sum(GameTime, Mul(PI, Float(1.5))), Index)), Sum(Float(0.4), Mul(Index, Float(0.05)))),
            ),
            shape_type: Line(
                thickness: Float(1.0),
            ),
            fill: Solid(
                color: State(Color),
            ),
            alpha: Context(T),
        ),
        children: [],
        mapping: {
            T: Sin(Sum(GameTime, Index)),
        },
        count: 4,
    ),
    state: (
        history: {},
        birth: 0.0,
    ),
    statuses: [],
)
                        status
                            .clone()
                            .spawn(world)
                            .insert(VarState::default())
                            .set_parent(owner);
                    }
                }
            }
            Effect::SendEvent(event) => {
                event.clone().send_with_context(context.clone(), world);
            }
            Effect::RemoveLocalTrigger => {
                let target = context.target();
                let local_trigger = Status::collect_unit_statuses(target, world)
                    .into_iter()
                    .find(|e| {
                        world
                            .get::<Status>(*e)
                            .is_some_and(|s| s.name.eq(LOCAL_TRIGGER))
                    });
                if let Some(entity) = local_trigger {
                    VarState::get_mut(entity, world).set_int(VarName::Charges, 0);
                }
                VarState::get_mut(target, world)
                    .set_string(VarName::Description, "--removed--".into());
            }
            Effect::If(cond, th, el) => {
                if cond.get_bool(context, world)? {
                    ActionPlugin::action_push_front(th.deref().clone(), context.clone(), world);
                } else {
                    ActionPlugin::action_push_front(el.deref().clone(), context.clone(), world);
                }
            }
            Effect::Repeat(count, effect) => {
                let count = count.get_int(context, world)?;
                for _ in 0..count {
                    ActionPlugin::action_push_front(effect.deref().clone(), context.clone(), world);
                }
            }
        }
        Ok(())
    }

    pub fn get_inner(&self) -> Vec<&Self> {
        match self {
            Effect::Noop
            | Effect::Kill
            | Effect::FullCopy
            | Effect::RemoveLocalTrigger
            | Effect::Debug(..)
            | Effect::Text(..)
            | Effect::Damage(..)
            | Effect::UseAbility(..)
            | Effect::Summon(..)
            | Effect::AddStatus(..)
            | Effect::Vfx(..)
            | Effect::StateAddVar(..)
            | Effect::AbilityStateAddVar(..)
            | Effect::SendEvent(..) => default(),
            Effect::AoeFaction(_, e)
            | Effect::WithTarget(_, e)
            | Effect::Repeat(_, e)
            | Effect::WithOwner(_, e)
            | Effect::WithVar(_, _, e) => vec![e],
            Effect::If(_, t, e) => vec![t, e],
            Effect::List(list) | Effect::ListSpread(list) => {
                list.iter().map(|e| e.as_ref()).collect_vec()
            }
        }
    }

    pub fn find_all_abilities(&self) -> Vec<Self> {
        let mut result: Vec<Self> = default();
        let mut queue = VecDeque::from([self]);
        while let Some(e) = queue.pop_front() {
            if matches!(e, Effect::UseAbility(..)) {
                result.push(e.clone());
            }
            queue.extend(e.get_inner());
        }
        result
    }
}
