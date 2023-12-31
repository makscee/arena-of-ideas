use std::ops::Deref;

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
                    {
                        let context = context.clone().set_owner(target, world).take();
                        ActionCluster::current(world).push_var_change(
                            VarName::Hp,
                            VarChange::new(VarValue::Int(new_hp)),
                            context,
                        );
                    }
                    VarState::push_back(
                        target,
                        VarName::LastAttacker,
                        VarChange::new(VarValue::Entity(context.owner())),
                        world,
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
                VarState::change_int(target, VarName::Hp, -9999999, world)?;
                VarState::push_back(
                    target,
                    VarName::LastAttacker,
                    VarChange::new(VarValue::Entity(context.owner())),
                    world,
                );
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
                {
                    let mut context = context.clone();
                    if context.get_var(VarName::Charges, world).is_none() {
                        context.set_var(
                            VarName::Charges,
                            context.get_var(VarName::Stacks, world).unwrap(),
                        );
                    }
                    ActionCluster::current(world).push_action_front(effect, context);
                }
                Pools::get_vfx("text", world)
                    .clone()
                    .set_var(
                        VarName::Position,
                        VarState::get(context.owner(), world).get_value_last(VarName::Position)?,
                    )
                    .set_var(VarName::Text, VarValue::String(format!("Use {ability}")))
                    .set_var(
                        VarName::Color,
                        VarValue::Color(
                            Pools::get_ability_house(ability, world)
                                .with_context(|| {
                                    format!("Failed to find house for ability {ability}")
                                })?
                                .color
                                .clone()
                                .into(),
                        ),
                    )
                    .unpack(world)?;
                ActionCluster::current(world).incr_order();
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
                        VarValue::String(format!("+{status} x{charges}")),
                    )
                    .set_var(VarName::Color, VarValue::Color(color))
                    .unpack(world)?;
            }
            Effect::List(list) => {
                for effect in list {
                    ActionCluster::current(world)
                        .push_action_front(effect.deref().clone(), context.clone());
                }
            }
            Effect::ListSpread(list) => {
                for effect in list {
                    ActionPlugin::new_cluster(effect.deref().clone(), context.clone(), world);
                }
            }
            Effect::AoeFaction(faction, effect) => {
                for unit in UnitPlugin::collect_faction(faction.get_faction(context, world)?, world)
                {
                    let context = context.clone().set_target(unit, world).take();
                    ActionCluster::current(world)
                        .push_action_front(effect.deref().clone(), context);
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
                let context = context
                    .set_target(target.get_entity(context, world)?, world)
                    .clone();
                ActionCluster::current(world).push_action_front(effect.deref().clone(), context);
            }
            Effect::WithOwner(owner, effect) => {
                let context = context
                    .set_owner(owner.get_entity(context, world)?, world)
                    .clone();
                ActionCluster::current(world).push_action_front(effect.deref().clone(), context);
            }
            Effect::WithVar(var, value, effect) => {
                let context = context
                    .set_var(*var, value.get_value(context, world)?)
                    .clone();
                ActionCluster::current(world).push_action_front(effect.deref().clone(), context);
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
                        VarState::push_back(owner, var, VarChange::new(value), world);
                    }
                }
                if !SkipVisual::active(world) {
                    Representation::pack(target, world).unpack(None, Some(owner), world);
                }
                // let source = &world.get::<Unit>(target).unwrap().source;
                // source
                //     .representation
                //     .clone()
                //     .unpack(None, Some(owner), world);
                // if let Some(entity) = PackedUnit::get_representation_entity(owner, world) {
                //     world.get_entity_mut(entity).unwrap().despawn_recursive();
                // }
                for entity in Status::collect_entity_statuses(owner, world) {
                    world.entity_mut(entity).despawn_recursive();
                }
                for entity in Status::collect_entity_statuses(target, world) {
                    let status = world.get::<Status>(entity).unwrap();
                    if let Some(status) = Pools::get_status(&status.name, world) {
                        let status = status.clone().unpack(owner, world);
                        for (var, history) in
                            VarState::get(entity, world).history.clone().into_iter()
                        {
                            if let Some(value) = history.get_last() {
                                VarState::push_back(status, var, VarChange::new(value), world);
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
        }
        Ok(())
    }

    pub fn show_editor(
        &mut self,
        editing_data: &mut EditingData,
        path: String,
        ui: &mut Ui,
        world: &mut World,
    ) {
        let hovered = if let Some(hovered) = editing_data.hovered.as_ref() {
            hovered.eq(&path)
        } else {
            false
        };
        let color = match hovered {
            true => hex_color!("#FF9100"),
            false => hex_color!("#9575CD"),
        };
        ui.style_mut().visuals.hyperlink_color = color;
        let mut now_hovered = false;
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                let link = ui.link(RichText::new(format!("( {self}")));
                if link.clicked() {
                    editing_data.lookup.clear();
                    link.request_focus();
                }
                now_hovered |= link.hovered();
                if link.has_focus() || link.lost_focus() {
                    let mut need_clear = false;
                    ui.horizontal_wrapped(|ui| {
                        ui.label(editing_data.lookup.to_owned());
                        Effect::iter()
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
                                    *self = e;
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
                Effect::Noop | Effect::Kill | Effect::FullCopy => {}
                Effect::Debug(e) | Effect::Text(e) => {
                    e.show_editor(editing_data, format!("{path}/e"), ui);
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
                        e.show_editor(editing_data, format!("{path}/e"), ui);
                    }
                }
                Effect::AoeFaction(exp, e)
                | Effect::WithTarget(exp, e)
                | Effect::WithOwner(exp, e) => {
                    ui.vertical(|ui| {
                        exp.show_editor(editing_data, format!("{path}/exp"), ui);
                        e.show_editor(editing_data, format!("{path}/e"), ui, world);
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
                                e.show_editor(editing_data, format!("{path}/{i}"), ui, world);
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
                        exp.show_editor(editing_data, format!("{path}/exp"), ui);
                        e.show_editor(editing_data, format!("{path}/e"), ui, world);
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
            if now_hovered && !editing_data.hovered.as_ref().eq(&Some(&path)) {
                editing_data.hovered = Some(path.clone());
            }
        });
    }
}
