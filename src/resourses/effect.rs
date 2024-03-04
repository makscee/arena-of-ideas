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
    UseAbility(String),
    AddStatus(String),
    Vfx(String),
    SendEvent(Event),
    RemoveLocalTrigger,
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
                .send(world)
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
                    .send(world);
                    Event::DamageDealt {
                        owner,
                        target,
                        value,
                    }
                    .send(world);
                    Pools::get_vfx("pain", world)
                        .set_parent(context.target())
                        .unpack(world)?;
                }
                Pools::get_vfx("text", world)
                    .clone()
                    .set_var(
                        VarName::Position,
                        VarValue::Vec2(UnitPlugin::get_unit_position(context.target(), world)?),
                    )
                    .set_var(VarName::Text, VarValue::String(format!("-{value}")))
                    .set_var(VarName::Color, VarValue::Color(Color::ORANGE_RED))
                    .unpack(world)?;
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
            Effect::UseAbility(ability) => {
                let effect = Pools::get_ability(ability, world)
                    .with_context(|| format!("Ability not found {ability}"))?
                    .effect
                    .clone();
                let color = Pools::get_ability_house(ability, world)
                    .with_context(|| format!("Failed to find house for ability {ability}"))?
                    .color
                    .clone()
                    .into();
                Pools::get_vfx("text", world)
                    .clone()
                    .set_var(
                        VarName::Position,
                        VarState::get(context.owner(), world).get_value_last(VarName::Position)?,
                    )
                    .set_var(VarName::Text, VarValue::String(format!("Use {ability}")))
                    .set_var(VarName::Color, VarValue::Color(color))
                    .unpack(world)?;
                {
                    let context = context
                        .clone()
                        .set_var(
                            VarName::Charges,
                            context
                                .get_var(VarName::Level, world)
                                .unwrap_or(VarValue::Int(1)),
                        )
                        .set_var(VarName::Color, VarValue::Color(color))
                        .take();
                    ActionPlugin::action_push_front(effect, context, world);
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
                let targets = if let Ok(targets) = target.get_entity_list() {
                    targets
                } else {
                    vec![target.get_entity()?]
                };
                let delay = 0.2;
                for target in targets {
                    let context = context.set_target(target, world).clone();
                    ActionPlugin::action_push_front_with_dealy(
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
                    } else {
                        status
                            .clone()
                            .spawn(world)
                            .insert(VarState::default())
                            .set_parent(owner);
                    }
                }
            }
            Effect::SendEvent(event) => {
                event.clone().send(world);
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
        }
        Ok(())
    }

    pub fn get_inner_mut(&mut self) -> Vec<&mut Self> {
        match self {
            Effect::Noop
            | Effect::Kill
            | Effect::FullCopy
            | Effect::RemoveLocalTrigger
            | Effect::Debug(..)
            | Effect::Text(..)
            | Effect::Damage(..)
            | Effect::UseAbility(..)
            | Effect::AddStatus(..)
            | Effect::Vfx(..)
            | Effect::SendEvent(..) => default(),
            Effect::AoeFaction(_, e)
            | Effect::WithTarget(_, e)
            | Effect::WithOwner(_, e)
            | Effect::WithVar(_, _, e) => vec![e],
            Effect::List(list) | Effect::ListSpread(list) => {
                list.iter_mut().map(|e| e.as_mut()).collect_vec()
            }
        }
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
            | Effect::AddStatus(..)
            | Effect::Vfx(..)
            | Effect::SendEvent(..) => default(),
            Effect::AoeFaction(_, e)
            | Effect::WithTarget(_, e)
            | Effect::WithOwner(_, e)
            | Effect::WithVar(_, _, e) => vec![e],
            Effect::List(list) | Effect::ListSpread(list) => {
                list.iter().map(|e| e.as_ref()).collect_vec()
            }
        }
    }

    pub fn find_ability(&mut self) -> Option<&mut Self> {
        let mut queue = VecDeque::from([self]);
        while let Some(e) = queue.pop_front() {
            if matches!(e, Effect::UseAbility(..)) {
                return Some(e);
            }
            queue.extend(e.get_inner_mut());
        }
        None
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

    pub fn show_editor(
        &mut self,
        hovered: &mut Option<String>,
        lookup: &mut String,
        path: String,
        ui: &mut Ui,
        world: &mut World,
    ) {
        let is_hovered = if let Some(hovered) = hovered.as_ref() {
            hovered.eq(&path)
        } else {
            false
        };
        let color = match is_hovered {
            true => hex_color!("#FF9100"),
            false => hex_color!("#9575CD"),
        };
        ui.style_mut().visuals.hyperlink_color = color;
        let mut now_hovered = false;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                let link = ui.link(RichText::new(format!("( {self}")));
                if link.clicked() {
                    lookup.clear();
                    link.request_focus();
                }
                now_hovered |= link.hovered();
                if link.has_focus() || link.lost_focus() {
                    let mut need_clear = false;
                    ui.horizontal_wrapped(|ui| {
                        ui.label(lookup.to_owned());
                        Effect::iter()
                            .filter_map(|e| {
                                match e
                                    .to_string()
                                    .to_lowercase()
                                    .starts_with(lookup.to_lowercase().as_str())
                                {
                                    true => Some(e),
                                    false => None,
                                }
                            })
                            .for_each(|e| {
                                let button = ui.button(e.to_string());
                                if button.gained_focus() || button.clicked() {
                                    *self = e;
                                    need_clear = true;
                                }
                            })
                    });
                    if need_clear {
                        lookup.clear();
                    }
                }
            });

            match self {
                Effect::Noop | Effect::Kill | Effect::FullCopy | Effect::RemoveLocalTrigger => {}
                Effect::Debug(e) | Effect::Text(e) => {
                    e.show_editor(hovered, lookup, format!("{path}/e"), ui);
                }
                Effect::Damage(e) => {
                    let mut is_some = e.is_some();
                    if ui.checkbox(&mut is_some, "").changed() {
                        if is_some {
                            *e = Some(default());
                        } else {
                            *e = None;
                        }
                    }
                    if let Some(e) = e {
                        e.show_editor(hovered, lookup, format!("{path}/e"), ui);
                    }
                }
                Effect::AoeFaction(exp, e)
                | Effect::WithTarget(exp, e)
                | Effect::WithOwner(exp, e) => {
                    ui.vertical(|ui| {
                        exp.show_editor(hovered, lookup, format!("{path}/exp"), ui);
                        e.show_editor(hovered, lookup, format!("{path}/e"), ui, world);
                    });
                }
                Effect::List(list) | Effect::ListSpread(list) => {
                    ui.vertical(|ui| {
                        let mut delete: Option<usize> = None;
                        for (i, e) in list.iter_mut().enumerate() {
                            ui.horizontal(|ui| {
                                if ui.link("-").clicked() {
                                    delete = Some(i);
                                }
                                e.show_editor(hovered, lookup, format!("{path}/{i}"), ui, world);
                            });
                        }
                        if let Some(delete) = delete {
                            list.remove(delete);
                        }
                        if ui.button("+").clicked() {
                            list.push(default());
                        }
                    });
                }
                Effect::WithVar(var, exp, e) => {
                    ui.vertical(|ui| {
                        var.show_editor(ui);
                        exp.show_editor(hovered, lookup, format!("{path}/exp"), ui);
                        e.show_editor(hovered, lookup, format!("{path}/e"), ui, world);
                    });
                }
                Effect::AddStatus(name) | Effect::Vfx(name) => {
                    ui.text_edit_singleline(name);
                }
                Effect::UseAbility(name) => {
                    ComboBox::from_id_source(format!("{path}/ability"))
                        .selected_text(name.clone())
                        .show_ui(ui, |ui| {
                            for ability in Pools::get(world).abilities.keys() {
                                ui.selectable_value(name, ability.to_owned(), ability);
                            }
                        });
                }
                Effect::SendEvent(event) => {
                    ComboBox::from_id_source(format!("{path}/event"))
                        .selected_text(event.to_string())
                        .show_ui(ui, |ui| {
                            for new in [Event::BattleStart, Event::TurnEnd, Event::TurnStart] {
                                ui.selectable_value(event, new.clone(), new.to_string());
                            }
                        });
                }
            }
            ui.style_mut().visuals.hyperlink_color = color;
            let right = ui.link(RichText::new(")"));
            now_hovered |= right.hovered();
            if now_hovered && !hovered.as_ref().eq(&Some(&path)) {
                *hovered = Some(path.clone());
            }
        });
    }
}
