use convert_case::{Case, Casing};
use event::Event;
use std::{collections::VecDeque, ops::Deref};

use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, EnumIter, AsRefStr)]
pub enum Effect {
    #[default]
    Noop,
    Kill,
    FullCopy,
    Debug(Expression),
    Text(Expression),
    Damage(Option<Expression>),
    Heal(Option<Expression>),
    WithTarget(Expression, Box<Effect>),
    WithOwner(Expression, Box<Effect>),
    List(Vec<Box<Effect>>),
    ListSpread(Vec<Box<Effect>>),
    WithVar(VarName, Expression, Box<Effect>),
    StateSetVar(VarName, Expression, Expression),
    StateAddVar(VarName, Expression, Expression),
    AbilityStateAddVar(String, VarName, Expression),
    UseAbility(String, i32),
    Summon(String, Option<Box<Effect>>),
    AddStatus(String),
    AddAllStatuses,
    StealStatus(String),
    StealAllStatuses,
    ClearStatus(String),
    ClearAllStatuses,
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
                let target = context.get_target()?;
                let owner = context.get_owner()?;
                let mut value = match value {
                    Some(value) => value.get_value(context, world)?,
                    None => context.get_var(VarName::Pwr, world)?,
                };
                debug!("Damage {} {target:?}", value.to_string());
                let event = Event::IncomingDamage {
                    owner: target,
                    value: value.get_int()?,
                };
                event.clone().send_with_context(context.clone(), world);
                event.map(&mut value, world);
                debug!("Value after map {value:?}");
                let value = value.get_int()?;
                if value > 0 {
                    let new_dmg = VarState::get(target, world).get_int(VarName::Dmg)? + value;
                    VarState::get_mut(target, world)
                        .push_back(VarName::Dmg, VarChange::new(VarValue::Int(new_dmg)));
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
                        .set_parent(target)
                        .unpack(world)?;
                }
                let value = value.max(0);
                TextColumn::add(target, &format!("-{value}"), orange(), world)?;
                Pools::get_vfx("damage", world)
                    .attach_context(context, world)
                    .unpack(world)?;
            }
            Effect::Heal(value) => {
                let target = context.get_target()?;
                let value = match value {
                    Some(value) => value.get_value(context, world)?,
                    None => context.get_var(VarName::Pwr, world)?,
                };
                debug!("Heal {} {target:?}", value.to_string());
                let value = value.get_int()?;
                if value > 0 {
                    let new_dmg =
                        (VarState::get(target, world).get_int(VarName::Dmg)? - value).max(0);
                    VarState::get_mut(target, world)
                        .push_back(VarName::Dmg, VarChange::new(VarValue::Int(new_dmg)));
                    Pools::get_vfx("pleasure", world)
                        .set_parent(target)
                        .unpack(world)?;
                }
                let value = value.max(0);
                TextColumn::add(target, &format!("+{value}"), green(), world)?;
                Pools::get_vfx("heal", world)
                    .attach_context(context, world)
                    .unpack(world)?;
            }
            Effect::Kill => {
                let target = context.get_target()?;
                VarState::get_mut(target, world)
                    .push_back(
                        VarName::LastAttacker,
                        VarChange::new(VarValue::Entity(context.owner())),
                    )
                    .change_int(VarName::Dmg, 9999999);
                TextColumn::add(target, "Kill", red(), world)?;
            }
            Effect::Debug(msg) => {
                let msg = msg.get_string(context, world)?;
                debug!("Debug effect: {msg}");
            }
            Effect::Noop => {}
            Effect::UseAbility(ability, base) => {
                let effect = Pools::get_ability(ability, world)
                    .with_context(|| format!("Ability not found {ability}"))?
                    .effect
                    .clone();
                let color = Pools::get_color_by_name(ability, world)?;
                let faction = context.get_faction(world)?;
                TeamPlugin::inject_ability_state(faction, ability, context, world);
                TextColumn::add_colored(
                    context.owner(),
                    "Use "
                        .add_color(white())
                        .push_colored(ability.add_color(color.c32()))
                        .set_style_ref(ColoredStringStyle::Bold)
                        .take(),
                    world,
                )?;

                let mut context = context
                    .clone()
                    .set_var(VarName::Color, VarValue::Color(color))
                    .take();
                let charges = context
                    .get_var(VarName::Level, world)
                    .map(|v| v.get_int().unwrap())
                    .unwrap_or(1)
                    + *base;
                let caster = context.owner();
                context
                    .set_var(VarName::Charges, VarValue::Int(charges))
                    .set_caster(caster, world);
                ActionPlugin::action_push_front(effect, context, world);
            }
            Effect::Summon(name, then) => {
                let mut unit = Pools::get_summon(name, world)
                    .with_context(|| format!("Summon unit not found {name}"))?
                    .clone();
                let faction = context.get_faction(world)?;
                if UnitPlugin::collect_faction(faction, world).len() > UNITS_LIMIT {
                    return Err(anyhow!("Units limit {UNITS_LIMIT} reached"));
                }
                TeamPlugin::inject_ability_state(faction, name, context, world);
                let extra_hp = context
                    .get_ability_var(name, VarName::Hp)
                    .unwrap_or(VarValue::Int(0))
                    .get_int()?;
                let extra_atk = context
                    .get_ability_var(name, VarName::Pwr)
                    .unwrap_or(VarValue::Int(0))
                    .get_int()?;
                unit.hp += extra_hp;
                unit.pwr += extra_atk;

                let color = Pools::get_color_by_name(name, world)?;

                let mut context = context
                    .clone()
                    .set_var(VarName::Color, VarValue::Color(color))
                    .take();
                if context.get_var(VarName::Charges, world).is_err() {
                    context.set_var(
                        VarName::Charges,
                        context
                            .get_var(VarName::Level, world)
                            .unwrap_or(VarValue::Int(1)),
                    );
                }
                let faction = context.get_faction(world)?;
                let parent = TeamPlugin::find_entity(faction, world).context("Team not found")?;
                let entity = unit.unpack(parent, None, world);
                if let Some(then) = then {
                    ActionPlugin::action_push_front(
                        *then.clone(),
                        context.clone().set_target(entity, world).take(),
                        world,
                    );
                }
                UnitPlugin::fill_slot_gaps(faction, world);
                UnitPlugin::translate_to_slots(world);
                Event::Summon(entity).send_with_context(
                    context.clone().set_caster(context.owner(), world).take(),
                    world,
                );
            }
            Effect::AddStatus(status) => {
                let charges = context
                    .get_var(VarName::Charges, world)
                    .unwrap_or(VarValue::Int(1))
                    .get_int()?;
                let color = Pools::get_color_by_name(status, world)?;
                let target = context.get_target()?;
                Status::change_charges(status, target, charges, world)?;

                TextColumn::add_colored(
                    target,
                    format!("{status} ")
                        .add_color(color.c32())
                        .push(
                            match charges.is_positive() {
                                true => format!("+{charges}"),
                                false => charges.to_string(),
                            },
                            white(),
                        )
                        .set_style_ref(ColoredStringStyle::Bold)
                        .take(),
                    world,
                )?;
                if charges > 0 {
                    Pools::get_vfx("apply_status", world)
                        .attach_context(
                            &context
                                .clone()
                                .set_var(VarName::Color, VarValue::Color(color)),
                            world,
                        )
                        .unpack(world)?;
                }
            }
            Effect::AddAllStatuses => {
                let target = context.get_target()?;
                let charges = context
                    .get_var(VarName::Charges, world)
                    .unwrap_or(VarValue::Int(1))
                    .get_int()?;
                let polarity = context
                    .get_var(VarName::Polarity, world)
                    .and_then(|v| v.get_int())
                    .ok();
                for (status, _) in Status::collect_statuses_name_charges(
                    target,
                    polarity,
                    GameTimer::get().insert_head(),
                    world,
                ) {
                    ActionPlugin::action_push_front(
                        Self::AddStatus(status),
                        context
                            .clone()
                            .set_var(VarName::Charges, VarValue::Int(charges))
                            .take(),
                        world,
                    )
                }
            }
            Effect::StealStatus(status) => {
                let target = context.get_target()?;
                let owner = context.owner();
                let charges = context
                    .get_var(VarName::Charges, world)
                    .unwrap_or(VarValue::Int(1))
                    .get_int()?
                    .min(Status::get_status_charges(target, status, world)?);
                if charges <= 0 {
                    return Err(anyhow!("Can't steal nonpositive charges amount"));
                }
                let color = Pools::get_color_by_name(status, world)?;
                Status::change_charges(status, target, -charges, world)?;
                Status::change_charges(status, context.owner(), charges, world)?;
                let text = "steal "
                    .add_color(white())
                    .push(format!("{status} "), color.c32())
                    .push(format!("-{charges}"), white())
                    .set_style_ref(ColoredStringStyle::Bold)
                    .take();
                TextColumn::add_colored(target, text, world)?;
                Pools::get_vfx("apply_status", world)
                    .attach_context(
                        &context
                            .clone()
                            .set_target(owner, world)
                            .set_owner(target, world)
                            .set_var(VarName::Color, VarValue::Color(color)),
                        world,
                    )
                    .unpack(world)?;
            }
            Effect::StealAllStatuses => {
                let target = context.get_target()?;
                let charges = context
                    .get_var(VarName::Charges, world)
                    .unwrap_or(VarValue::Int(1))
                    .get_int()?;
                if charges <= 0 {
                    return Err(anyhow!("Can't steal nonpositive charges amount"));
                }
                let polarity = context
                    .get_var(VarName::Polarity, world)
                    .and_then(|v| v.get_int())
                    .ok();
                for (status, c) in Status::collect_statuses_name_charges(
                    target,
                    polarity,
                    GameTimer::get().insert_head(),
                    world,
                ) {
                    ActionPlugin::action_push_front(
                        Self::StealStatus(status),
                        context
                            .clone()
                            .set_var(VarName::Charges, VarValue::Int(charges.min(c)))
                            .take(),
                        world,
                    )
                }
            }
            Effect::ClearStatus(status) => {
                let target = context.get_target()?;
                let charges = Status::get_status_charges(target, status, world)?;
                if charges <= 0 {
                    return Err(anyhow!("Charges <= 0: {status} ({charges})"));
                }
                let color = Pools::get_color_by_name(status, world)?;
                Status::change_charges(status, target, -charges, world)?;
                TextColumn::add_colored(
                    target,
                    "Clear"
                        .add_color(white())
                        .push_colored(status.add_color(color.c32()))
                        .set_style_ref(ColoredStringStyle::Bold)
                        .take(),
                    world,
                )?;
            }
            Effect::ClearAllStatuses => {
                let target = context.get_target()?;
                let charges = context
                    .get_var(VarName::Charges, world)
                    .unwrap_or(VarValue::Int(1))
                    .get_int()?;
                if charges <= 0 {
                    return Err(anyhow!("Can't clear nonpositive charges amount"));
                }
                let polarity = context
                    .get_var(VarName::Polarity, world)
                    .and_then(|v| v.get_int())
                    .ok();
                for (status, c) in Status::collect_statuses_name_charges(
                    target,
                    polarity,
                    GameTimer::get().insert_head(),
                    world,
                ) {
                    ActionPlugin::action_push_front(
                        Self::AddStatus(status),
                        context
                            .clone()
                            .set_var(VarName::Charges, VarValue::Int(-charges.min(c)))
                            .take(),
                        world,
                    )
                }
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
            Effect::Text(text) => {
                let text = text.get_string(context, world)?;
                TextColumn::add(
                    context.get_owner()?,
                    &text,
                    context.get_var(VarName::Color, world)?.get_color()?.c32(),
                    world,
                )?;
            }
            Effect::Vfx(name) => {
                Pools::get_vfx(name, world)
                    .clone()
                    .attach_context(context, world)
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
            Effect::StateSetVar(var, target, value) => {
                let target = target.get_entity(context, world)?;
                let value = value.get_value(context, world)?;
                let mut state = VarState::try_get_mut(target, world)?;
                state.push_back(*var, VarChange::new(value));
            }
            Effect::StateAddVar(var, target, value) => {
                let target = target.get_entity(context, world)?;
                let value = value.get_value(context, world).unwrap_or(VarValue::Int(0));
                let mut state = VarState::try_get_mut(target, world)?;
                let value = match state.get_value_last(*var) {
                    Ok(prev) => VarValue::sum(&value, &prev)?,
                    Err(_) => value,
                };
                state.push_back(*var, VarChange::new(value));
            }
            Effect::AbilityStateAddVar(ability, var, value) => {
                let value = value.get_value(context, world)?;
                let faction = context.get_faction(world)?;
                let text = format!("{ability} {var} add {value}");
                TeamPlugin::add_ability_var(faction, ability, *var, value, world)?;
                let color = Pools::get_color_by_name(ability, world)?;
                TextColumn::add(context.owner(), &text, color.c32(), world)?;
            }
            Effect::FullCopy => {
                let owner = context.owner();
                let target = context.get_target()?;
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
                    Representation::pack(target, world)
                        .unpack(world.spawn_empty().set_parent(owner).id(), world);
                }
                for entity in Status::collect_unit_statuses(owner, world) {
                    let mut state = VarState::get_mut(entity, world);
                    if state.get_int(VarName::Charges).is_ok_and(|v| v > 0) {
                        state.set_int(VarName::Charges, 0);
                    }
                }
                for target_status in Status::collect_unit_statuses(target, world) {
                    let status = world.get::<Status>(target_status).unwrap().clone();
                    if Pools::get_status(&status.name, world).is_some() {
                        let delta =
                            VarState::get(target_status, world).get_int(VarName::Charges)?;
                        let name = status.name;
                        Status::change_charges(&name, owner, delta, world)?;
                    } else {
                        if let Some((entity, mut old_status)) =
                            Status::find_unit_status(owner, LOCAL_TRIGGER, world)
                        {
                            *old_status = status.clone();
                            let state = VarState::get(target_status, world).final_snapshot();
                            world.entity_mut(entity).insert(state);
                        }
                    }
                }
            }
            Effect::SendEvent(event) => {
                event.clone().send_with_context(context.clone(), world);
            }
            Effect::RemoveLocalTrigger => {
                let target = context.get_target()?;
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
            | Effect::Heal(..)
            | Effect::UseAbility(..)
            | Effect::Summon(..)
            | Effect::AddStatus(..)
            | Effect::StealStatus(..)
            | Effect::StealAllStatuses
            | Effect::AddAllStatuses
            | Effect::ClearStatus(..)
            | Effect::ClearAllStatuses
            | Effect::Vfx(..)
            | Effect::StateSetVar(..)
            | Effect::StateAddVar(..)
            | Effect::AbilityStateAddVar(..)
            | Effect::SendEvent(..) => default(),
            Effect::WithTarget(_, e)
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

impl EditorNodeGenerator for Effect {
    fn node_color(&self) -> Color32 {
        white()
    }

    fn show_extra(&mut self, path: &str, context: &Context, world: &mut World, ui: &mut Ui) {
        match self {
            Effect::WithTarget(_, _)
            | Effect::WithOwner(_, _)
            | Effect::Noop
            | Effect::Kill
            | Effect::FullCopy
            | Effect::RemoveLocalTrigger
            | Effect::StealAllStatuses
            | Effect::ClearAllStatuses
            | Effect::AddAllStatuses
            | Effect::Debug(_)
            | Effect::Text(_) => {}

            Effect::List(list) | Effect::ListSpread(list) => {
                if ui.button("CLEAR").clicked() {
                    list.clear()
                }
            }
            Effect::Damage(e) | Effect::Heal(e) => {
                let mut v = e.is_some();
                if ui.checkbox(&mut v, "").changed() {
                    *e = match v {
                        true => Some(default()),
                        false => None,
                    };
                }
            }
            Effect::WithVar(x, e, _) => {
                ui.vertical(|ui| {
                    x.show_editor(path, ui);
                    let value = e.get_value(context, world);
                    show_value(&value, ui);
                });
            }
            Effect::If(e, ..) | Effect::Repeat(e, ..) => {
                ui.vertical(|ui| {
                    let value = e.get_value(context, world);
                    show_value(&value, ui);
                });
            }
            Effect::StateAddVar(x, target, value) | Effect::StateSetVar(x, target, value) => {
                ui.vertical(|ui| {
                    x.show_editor(path, ui);
                    ui.horizontal(|ui| {
                        ui.label("target:");
                        let target = target.get_value(context, world);
                        show_value(&target, ui);
                    });
                    ui.horizontal(|ui| {
                        ui.label("value:");
                        let value = value.get_value(context, world);
                        show_value(&value, ui);
                    });
                });
            }
            Effect::AbilityStateAddVar(name, x, value) => {
                ui.vertical(|ui| {
                    ComboBox::from_id_source(Id::new(path).with("ability"))
                        .selected_text(name.to_owned())
                        .show_ui(ui, |ui| {
                            let names = {
                                let pools = Pools::get(world);
                                pools
                                    .abilities
                                    .keys()
                                    .chain(pools.statuses.keys())
                                    .chain(pools.summons.keys())
                                    .unique()
                                    .sorted()
                            };
                            for option in names {
                                let text = option.to_string();
                                ui.selectable_value(name, option.to_owned(), text);
                            }
                        });
                    x.show_editor(Id::new(path).with("var"), ui);
                    ui.horizontal(|ui| {
                        ui.label("value:");
                        let value = value.get_value(context, world);
                        show_value(&value, ui);
                    });
                });
            }
            Effect::UseAbility(name, base) => {
                ui.vertical(|ui| {
                    ComboBox::from_id_source(&path)
                        .selected_text(name.to_owned())
                        .show_ui(ui, |ui| {
                            let colors = &Pools::get(world).colors;
                            for option in Pools::get(world).abilities.keys().sorted() {
                                let text = option
                                    .to_string()
                                    .add_color(
                                        colors.get(option).map(|c| c.c32()).unwrap_or(light_gray()),
                                    )
                                    .rich_text(ui);
                                ui.selectable_value(name, option.to_owned(), text);
                            }
                        });
                    DragValue::new(base).ui(ui);
                });
            }
            Effect::Summon(name, then) => {
                ui.vertical(|ui| {
                    ComboBox::from_id_source(&path)
                        .selected_text(name.to_owned())
                        .show_ui(ui, |ui| {
                            for option in Pools::get(world).summons.keys().sorted() {
                                let text = option.to_string();
                                ui.selectable_value(name, option.to_owned(), text);
                            }
                        });
                    let mut v = then.is_some();
                    if ui.checkbox(&mut v, "").changed() {
                        *then = match v {
                            true => Some(default()),
                            false => None,
                        };
                    }
                });
            }
            Effect::AddStatus(name) | Effect::ClearStatus(name) | Effect::StealStatus(name) => {
                Status::show_selector(name, path, ui, world);
            }
            Effect::Vfx(name) => {
                ui.vertical(|ui| {
                    ComboBox::from_id_source(&path)
                        .selected_text(name.to_owned())
                        .show_ui(ui, |ui| {
                            for option in Pools::get(world).vfx.keys().sorted() {
                                let text = option.to_string();
                                ui.selectable_value(name, option.to_owned(), text);
                            }
                        });
                });
            }
            Effect::SendEvent(name) => {
                ui.vertical(|ui| {
                    ComboBox::from_id_source(&path)
                        .selected_text(name.to_string())
                        .show_ui(ui, |ui| {
                            for option in [Event::BattleStart, Event::TurnStart, Event::TurnEnd] {
                                let text = option.to_string();
                                ui.selectable_value(name, option, text);
                            }
                        });
                });
            }
        }
    }

    fn show_replace_buttons(&mut self, lookup: &str, submit: bool, ui: &mut Ui) -> bool {
        for e in Effect::iter() {
            if e.as_ref().to_lowercase().contains(lookup) {
                let btn = e.as_ref().add_color(e.node_color()).rich_text(ui);
                let btn = ui.button(btn);
                if btn.clicked() || submit {
                    btn.request_focus();
                }
                if btn.gained_focus() {
                    *self = e;
                    return true;
                }
            }
        }
        false
    }

    fn show_children(
        &mut self,
        path: &str,
        connect_pos: Option<Pos2>,
        context: &Context,
        ui: &mut Ui,
        world: &mut World,
    ) {
        match self {
            Effect::Noop
            | Effect::Kill
            | Effect::FullCopy
            | Effect::UseAbility(..)
            | Effect::AddStatus(..)
            | Effect::StealStatus(..)
            | Effect::StealAllStatuses
            | Effect::ClearAllStatuses
            | Effect::AddAllStatuses
            | Effect::ClearStatus(..)
            | Effect::Vfx(..)
            | Effect::SendEvent(..)
            | Effect::RemoveLocalTrigger
            | Effect::Debug(..) => {}

            Effect::Text(e) | Effect::AbilityStateAddVar(_, _, e) => {
                show_node(e, format!("{path}:e"), connect_pos, context, ui, world)
            }
            Effect::Damage(e) | Effect::Heal(e) => {
                if let Some(e) = e {
                    show_node(e, format!("{path}:e"), connect_pos, context, ui, world);
                }
            }
            Effect::Summon(_, then) => {
                if let Some(then) = then {
                    show_node(
                        &mut **then,
                        format!("{path}:then"),
                        connect_pos,
                        context,
                        ui,
                        world,
                    );
                }
            }
            Effect::WithTarget(e, eff) | Effect::WithOwner(e, eff) | Effect::Repeat(e, eff) => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        show_node(e, format!("{path}:e"), connect_pos, context, ui, world);
                    });
                    ui.horizontal(|ui| {
                        show_node(
                            eff.as_mut(),
                            format!("{path}:eff"),
                            connect_pos,
                            context,
                            ui,
                            world,
                        );
                    });
                });
            }
            Effect::List(list) => {
                ui.vertical(|ui| {
                    for (i, eff) in list.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            show_node(
                                eff.as_mut(),
                                format!("{path}:eff{i}"),
                                connect_pos,
                                context,
                                ui,
                                world,
                            );
                        });
                    }
                    if ui.button("+").clicked() {
                        list.push(default());
                    }
                });
            }
            Effect::ListSpread(_) => todo!(),
            Effect::WithVar(_, e, eff) => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        show_node(e, format!("{path}:e"), connect_pos, context, ui, world);
                    });
                    ui.horizontal(|ui| {
                        show_node(
                            eff.as_mut(),
                            format!("{path}:eff"),
                            connect_pos,
                            context,
                            ui,
                            world,
                        );
                    });
                });
            }
            Effect::StateAddVar(_, target, value) | Effect::StateSetVar(_, target, value) => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        show_node(
                            target,
                            format!("{path}:target"),
                            connect_pos,
                            context,
                            ui,
                            world,
                        );
                    });
                    ui.horizontal(|ui| {
                        show_node(
                            value,
                            format!("{path}:value"),
                            connect_pos,
                            context,
                            ui,
                            world,
                        );
                    });
                });
            }
            Effect::If(cond, th, el) => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        show_node(
                            cond,
                            format!("{path}:cond"),
                            connect_pos,
                            context,
                            ui,
                            world,
                        );
                    });
                    ui.horizontal(|ui| {
                        show_node(
                            th.as_mut(),
                            format!("{path}:then"),
                            connect_pos,
                            context,
                            ui,
                            world,
                        );
                    });
                    ui.horizontal(|ui| {
                        show_node(
                            el.as_mut(),
                            format!("{path}:else"),
                            connect_pos,
                            context,
                            ui,
                            world,
                        );
                    });
                });
            }
        };
    }

    fn wrap(&mut self) {
        *self = Effect::List([Box::new(self.clone())].into());
    }

    fn show_context_menu(&mut self, _: &mut Ui) {}
}

impl std::fmt::Display for Effect {
    fn fmt(&self, f: &mut __private::Formatter<'_>) -> std::fmt::Result {
        match self {
            Effect::RemoveLocalTrigger
            | Effect::StealAllStatuses
            | Effect::ClearAllStatuses
            | Effect::AddAllStatuses
            | Effect::FullCopy
            | Effect::Kill
            | Effect::Noop => {
                write!(f, "{}", self.as_ref().to_case(Case::Lower))
            }
            Effect::Text(x) | Effect::Debug(x) => {
                write!(f, "{}({x})", self.as_ref())
            }
            Effect::Damage(x) | Effect::Heal(x) => write!(
                f,
                "{}({})",
                self.as_ref(),
                x.as_ref()
                    .and_then(|x| Some(x.to_string()))
                    .unwrap_or_default()
            ),
            Effect::WithOwner(x, e) | Effect::WithTarget(x, e) => {
                write!(f, "{} ({x}, {e})", self.as_ref())
            }
            Effect::List(list) | Effect::ListSpread(list) => write!(
                f,
                "({})",
                list.into_iter().map(|e| e.to_string()).join(" & ")
            ),
            Effect::WithVar(v, x, e) => write!(f, "{} ({v} -> {x}, {e})", self.as_ref()),
            Effect::StateAddVar(var, t, v) | Effect::StateSetVar(var, t, v) => {
                write!(f, "{} {t} ({var} -> {v})", self.as_ref())
            }
            Effect::AbilityStateAddVar(ability, var, v) => {
                write!(f, "[{ability}]: {var} add {v}")
            }
            Effect::UseAbility(name, base) => write!(
                f,
                "use [{name}] ({{Level}}{})",
                if *base > 0 {
                    format!("+{base}")
                } else {
                    default()
                }
            ),
            Effect::SendEvent(name) => write!(f, "{} ({name})", self.as_ref()),
            Effect::Vfx(name)
            | Effect::ClearStatus(name)
            | Effect::AddStatus(name)
            | Effect::StealStatus(name) => {
                write!(f, "{} [{name}]", self.as_ref().to_case(Case::Lower))
            }
            Effect::Summon(name, then) => {
                write!(
                    f,
                    "{} [{name}]{}",
                    self.as_ref().to_case(Case::Lower),
                    then.as_ref()
                        .and_then(|x| Some(format!(" -> {}", x)))
                        .unwrap_or_default()
                )
            }
            Effect::If(c, t, e) => write!(f, "{} {c} ({t} else {e})", self.as_ref()),
            Effect::Repeat(c, e) => write!(f, "{} ({e}) x {c}", self.as_ref()),
        }
    }
}
