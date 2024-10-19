use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, AsRefStr, EnumIter)]
pub enum Effect {
    #[default]
    Noop,
    Damage,
    Kill,
    Heal,
    ChangeStatus(String),
    ClearStatus(String),
    StealStatus(String),
    ChangeAllStatuses,
    ClearAllStatuses,
    StealAllStatuses,
    UseAbility(String, i32),
    AbilityStateAddVar(String, VarName, Expression),
    Summon(String, Option<Box<Effect>>),
    WithTarget(Expression, Box<Effect>),
    WithOwner(Expression, Box<Effect>),
    WithVar(VarName, Expression, Box<Effect>),
    List(Vec<Box<Effect>>),
    Repeat(Expression, Box<Effect>),
    If(Expression, Box<Effect>, Box<Effect>),
    Vfx(String),
    StateAddVar(VarName, Expression, Expression),
    StatusSetVar(Expression, String, VarName, Expression),
    Text(Expression),
    FullCopy,
}

impl Effect {
    pub fn invoke(&self, context: &mut Context, world: &mut World) -> Result<()> {
        match self {
            Effect::Noop
            | Effect::Damage
            | Effect::Kill
            | Effect::Heal
            | Effect::FullCopy
            | Effect::ChangeStatus(..)
            | Effect::ClearStatus(..)
            | Effect::StealStatus(..)
            | Effect::ChangeAllStatuses
            | Effect::ClearAllStatuses
            | Effect::StealAllStatuses
            | Effect::UseAbility(..)
            | Effect::AbilityStateAddVar(..)
            | Effect::Summon(..)
            | Effect::Text(..)
            | Effect::StateAddVar(..)
            | Effect::StatusSetVar(..)
            | Effect::Vfx(..) => context.log(Some(
                "Invoke: "
                    .cstr_c(YELLOW)
                    .push(
                        self.cstr_expanded()
                            .inject_context(context, world)
                            .push("\n".cstr())
                            .take(),
                    )
                    .take(),
            )),
            Effect::WithTarget(..)
            | Effect::WithOwner(..)
            | Effect::WithVar(..)
            | Effect::List(..)
            | Effect::Repeat(..)
            | Effect::If(..) => {}
        };
        context.set_effect(self.cstr_expanded());
        let owner = context.owner();
        match self {
            Effect::Noop => {}
            Effect::Damage => {
                let target = context.get_target()?;
                let mut value = context
                    .get_value(VarName::Value, world)
                    .unwrap_or(context.get_value(VarName::Pwr, world)?);
                let i_value = value.get_int()?;
                Event::IncomingDamage {
                    owner: target,
                    value: i_value,
                }
                .send_with_context(context.clone(), world)
                .map(&mut value, world);
                let i_value = value.get_int()?;
                if i_value > 0 {
                    debug!("deal {i_value} dmg to {target:?}");
                    let mut state = VarState::try_get_mut(target, world)?;
                    state.change_int(VarName::Dmg, i_value);
                    state.set_value(VarName::LastAttacker, owner.into());
                    state.animate_float(VarName::Pain, 1.0, 0.0, client_settings().animation_time);
                    Event::DamageTaken {
                        owner: target,
                        value: i_value,
                    }
                    .send_with_context(context.clone(), world);
                    Event::DamageDealt {
                        owner,
                        target,
                        value: i_value,
                    }
                    .send_with_context(context.clone(), world);
                    Vfx::get("pain").set_parent(target).unpack(world)?;
                }
                TextColumnPlugin::add(
                    target,
                    format!("-{}", i_value.at_least(0)).cstr_cs(RED, CstrStyle::Bold),
                    world,
                );
                Vfx::get("damage")
                    .attach_context(context, world)
                    .unpack(world)?;
            }
            Effect::Kill => {
                let target = context.get_target()?;
                let mut state = VarState::try_get_mut(target, world)?;
                state.set_int(VarName::Dmg, i32::MAX / 2);
                Vfx::get("pain").set_parent(target).unpack(world)?;
                Vfx::get("damage")
                    .attach_context(context, world)
                    .unpack(world)?;
            }
            Effect::Heal => {
                let target = context.get_target()?;
                let value = context
                    .get_value(VarName::Value, world)
                    .unwrap_or(context.get_value(VarName::Pwr, world)?);
                let i_value = value.get_int()?;
                if i_value > 0 {
                    let dmg = Context::new(target).get_int(VarName::Dmg, world)?;
                    if dmg > 0 {
                        Vfx::get("pleasure").set_parent(target).unpack(world)?;
                    }
                    let dmg = (dmg - i_value).max(0);
                    VarState::get_mut(target, world).set_int(VarName::Dmg, dmg);
                    TextColumnPlugin::add(
                        target,
                        format!("+{i_value}").cstr_cs(GREEN, CstrStyle::Bold),
                        world,
                    );
                }
            }
            Effect::ChangeStatus(name) => {
                let delta = context.get_charges(world).unwrap_or(1);
                Status::change_charges_with_text(name, context.get_target()?, delta, world);
            }
            Effect::ChangeAllStatuses => {
                let target = context.get_target()?;
                let polarity = context
                    .get_value(VarName::Polarity, world)
                    .and_then(|v| v.get_int())
                    .ok();
                for (s, _) in
                    VarState::get(target, world).all_active_statuses_at(polarity, context.t())
                {
                    ActionPlugin::action_push_front(
                        Effect::ChangeStatus(s),
                        context.clone(),
                        world,
                    );
                }
            }
            Effect::ClearStatus(name) => {
                let target = context.get_target()?;
                let charges = context.get_charges(world).unwrap_or(1);
                let charges = charges.at_most(Status::get_charges(name, target, world)?);
                if charges <= 0 {
                    return Err(anyhow!("Status {name} is absent (c: {charges})"));
                }
                Status::change_charges_with_text(name, target, -charges, world);
            }
            Effect::ClearAllStatuses => {
                let target = context.get_target()?;
                let polarity = context
                    .get_value(VarName::Polarity, world)
                    .and_then(|v| v.get_int())
                    .ok();
                for (s, _) in
                    VarState::get(target, world).all_active_statuses_at(polarity, context.t())
                {
                    ActionPlugin::action_push_front(Effect::ClearStatus(s), context.clone(), world);
                }
            }
            Effect::StealStatus(name) => {
                let target = context.get_target()?;
                let charges = context.get_charges(world).unwrap_or(1);
                if charges <= 0 {
                    return Err(anyhow!("Can't steal nonpositive charges amount"));
                }
                let c = Status::get_charges(name, target, world)?;
                let delta = c.at_most(charges);
                Status::change_charges_with_text(name, target, -delta, world);
                Status::change_charges_with_text(name, owner, delta, world);
            }
            Effect::StealAllStatuses => {
                let target = context.get_target()?;
                let polarity = context
                    .get_value(VarName::Polarity, world)
                    .and_then(|v| v.get_int())
                    .ok();
                for (s, _) in
                    VarState::get(target, world).all_active_statuses_at(polarity, context.t())
                {
                    ActionPlugin::action_push_front(Effect::StealStatus(s), context.clone(), world);
                }
            }
            Effect::UseAbility(name, base) => {
                let ability = game_assets()
                    .abilities
                    .get(name)
                    .cloned()
                    .with_context(|| format!("Ability not found {name}"))
                    .unwrap();
                let charges = context
                    .get_value(VarName::Lvl, world)
                    .map(|v| v.get_int().unwrap())
                    .unwrap_or(0)
                    + *base;
                let caster = owner;
                let context = context
                    .clone()
                    .set_ability_state(name, world)?
                    .set_var(VarName::Charges, VarValue::Int(charges))
                    .set_caster(caster)
                    .set_var(VarName::Color, name_color(name).into())
                    .take();
                ActionPlugin::action_push_front(ability.effect, context.clone(), world);
                let txt = if *base > 0 {
                    format!("{name} +{base}")
                } else {
                    name.clone()
                };
                TextColumnPlugin::add(
                    caster,
                    "use "
                        .cstr()
                        .push(txt.cstr_cs(name_color(name), CstrStyle::Bold))
                        .take(),
                    world,
                );
            }
            Effect::AbilityStateAddVar(name, var, delta) => {
                let delta = delta.get_int(context, world)?;
                TeamPlugin::change_ability_var_int(
                    name.clone(),
                    *var,
                    delta,
                    context.get_faction(world)?,
                    world,
                );
                TextColumnPlugin::add(
                    owner,
                    name.cstr_cs(name_color(name), CstrStyle::Bold)
                        .push(var.cstr_c(visible_bright()))
                        .push(format!("+{delta}").cstr_c(visible_light()))
                        .join(&" ".cstr())
                        .take(),
                    world,
                );
            }
            Effect::Summon(name, then) => {
                let mut unit = game_assets()
                    .summons
                    .get(name)
                    .with_context(|| format!("Summon {name} not found"))
                    .unwrap()
                    .clone();
                let faction = context.get_faction(world)?;
                context.set_ability_state(name, world)?;
                let extra_pwr = context
                    .get_ability_var(name, VarName::Pwr)
                    .unwrap_or_default()
                    .get_int()?;
                let extra_hp = context
                    .get_ability_var(name, VarName::Hp)
                    .unwrap_or_default()
                    .get_int()?;
                unit.pwr += extra_pwr;
                unit.hp += extra_hp;
                let unit = unit.unpack(TeamPlugin::entity(faction, world), None, None, world);
                UnitPlugin::fill_gaps_and_translate(world);
                if let Some(then) = then {
                    ActionPlugin::action_push_front(
                        *then.clone(),
                        context.clone().set_target(unit).take(),
                        world,
                    );
                }
                Event::Summon(unit)
                    .send_with_context(context.clone().set_caster(owner).take(), world);
            }
            Effect::WithTarget(target, effect) => {
                let target = target.get_value(context, world)?;
                let targets = target
                    .get_entity_list()?
                    .into_iter()
                    .sorted_by_key(|e| {
                        -Context::new(*e)
                            .get_int(VarName::Slot, world)
                            .unwrap_or_default()
                    })
                    .collect_vec();
                let delay = 0.2;
                for target in targets {
                    let context = context.set_target(target).clone();
                    ActionPlugin::action_push_front_with_delay(
                        *effect.clone(),
                        context,
                        delay,
                        world,
                    );
                }
            }
            Effect::WithOwner(owner, effect) => {
                let owner = owner.get_entity(context, world)?;
                ActionPlugin::action_push_front(
                    *effect.clone(),
                    context.set_owner(owner).take(),
                    world,
                );
            }
            Effect::WithVar(var, value, effect) => {
                let context = context
                    .set_var(*var, value.get_value(context, world)?)
                    .clone();
                ActionPlugin::action_push_front(*effect.clone(), context, world);
            }
            Effect::List(list) => {
                for effect in list.into_iter().rev() {
                    ActionPlugin::action_push_front(*effect.clone(), context.clone(), world);
                }
            }
            Effect::Repeat(count, effect) => {
                let count = count.get_int(context, world)?;
                for _ in 0..count {
                    ActionPlugin::action_push_front(*effect.clone(), context.clone(), world);
                }
            }
            Effect::If(cond, th, el) => {
                ActionPlugin::action_push_front(
                    if cond.get_bool(context, world)? {
                        th
                    } else {
                        el
                    }
                    .deref()
                    .clone(),
                    context.clone(),
                    world,
                );
            }
            Effect::Vfx(name) => {
                Vfx::get(name)
                    .attach_context(context, world)
                    .unpack(world)?;
            }
            Effect::StateAddVar(var, target, value) => {
                let target = target.get_entity(context, world)?;
                let value = value.get_value(context, world)?;
                let mut state = VarState::try_get_mut(target, world)?;
                let value = match state.get_key_value_last(default(), *var) {
                    Ok(prev) => VarValue::sum(&value, &prev)?,
                    Err(_) => value,
                };
                state.push_change(*var, default(), VarChange::new(value));
            }
            Effect::StatusSetVar(target, status, var, value) => {
                let target = target.get_entity(context, world)?;
                let value = value.get_value(context, world)?;
                VarState::try_get_mut(target, world)?
                    .get_status_mut(status)
                    .context("Status not found")?
                    .push_change(*var, default(), VarChange::new(value));
            }
            Effect::Text(text) => {
                let text = text.get_string(context, world)?;
                let color = context
                    .get_value(VarName::Color, world)
                    .and_then(|c| c.get_color())
                    .map(|c| c.c32())
                    .unwrap_or(visible_bright());
                let target = context.get_target().unwrap_or(owner);
                TextColumnPlugin::add(target, text.cstr_cs(color, CstrStyle::Bold), world);
            }
            Effect::FullCopy => {
                let target = context.get_target()?;
                let state = VarState::get(target, world);
                let mut vars = state.all_own_values();
                vars.remove(&VarName::Slot);
                vars.remove(&VarName::Position);
                let status_charges = state.all_active_statuses_at(None, context.t());
                let status_components = Status::collect_statuses(target, world);
                let mut state = VarState::get_mut(owner, world);
                for (var, value) in vars {
                    state.set_value(var, value);
                }
                if let Some((_, local_status)) = status_components
                    .into_iter()
                    .find(|(_, s)| s.name.eq(LOCAL_STATUS))
                {
                    if let Ok(own_local_status) =
                        Status::find_status_entity(owner, LOCAL_STATUS, world)
                    {
                        world.entity_mut(own_local_status).insert(local_status);
                    } else {
                        local_status.spawn(owner, world);
                    }
                }
                for (status, charges) in status_charges {
                    let own_charges =
                        Status::get_charges(&status, owner, world).unwrap_or_default();
                    Status::change_charges(&status, owner, charges - own_charges, world);
                }
            }
        }
        Ok(())
    }
}

impl ToCstr for Effect {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr_c(match self {
            Effect::Noop | Effect::Damage | Effect::Kill | Effect::Heal => visible_light(),
            Effect::ChangeStatus(_)
            | Effect::ClearStatus(_)
            | Effect::StealStatus(_)
            | Effect::ChangeAllStatuses
            | Effect::ClearAllStatuses
            | Effect::StealAllStatuses => CYAN,
            Effect::Text(_)
            | Effect::FullCopy
            | Effect::UseAbility(_, _)
            | Effect::AbilityStateAddVar(_, _, _)
            | Effect::Summon(_, _) => PURPLE,
            Effect::WithTarget(_, _)
            | Effect::WithOwner(_, _)
            | Effect::WithVar(_, _, _)
            | Effect::List(_)
            | Effect::Repeat(_, _)
            | Effect::If(_, _, _)
            | Effect::Vfx(_)
            | Effect::StateAddVar(_, _, _)
            | Effect::StatusSetVar(_, _, _, _) => YELLOW,
        })
    }
    fn cstr_expanded(&self) -> Cstr {
        match self {
            Effect::UseAbility(name, base) => {
                let mut c = "use "
                    .cstr_c(visible_light())
                    .push(name.cstr_cs(name_color(name), CstrStyle::Bold))
                    .push(" lvl.".cstr_cs(visible_dark(), CstrStyle::Small))
                    .push(VarName::Lvl.cstr_cs(visible_bright(), CstrStyle::Bold))
                    .take();
                if *base > 0 {
                    c.push(format!(" +{base}").cstr_cs(visible_light(), CstrStyle::Bold));
                }
                c
            }
            Effect::Summon(name, after) => {
                let mut c = "summon "
                    .cstr_c(visible_light())
                    .push(name.cstr_cs(name_color(name), CstrStyle::Bold))
                    .take();
                if let Some(after) = after {
                    c.push(" then ".cstr().push(after.cstr_expanded()).take());
                }
                c
            }
            Effect::AbilityStateAddVar(ability, var, value) => ability
                .cstr_cs(name_color(ability), CstrStyle::Bold)
                .push(var.to_string().cstr_c(visible_bright()))
                .push("add ".cstr_c(visible_light()))
                .join_char(' ')
                .push(value.cstr_cs(visible_bright(), CstrStyle::Bold))
                .take(),
            Effect::Vfx(name) => format!("Vfx({name})").cstr_c(visible_light()),
            Effect::List(l) => "List"
                .cstr()
                .push_wrapped_circ(
                    l.into_iter()
                        .map(|l| l.cstr_expanded())
                        .collect_vec()
                        .join(" + ".cstr()),
                )
                .take(),
            _ => self.as_ref().cstr_c(visible_light()),
        }
    }
}

impl ShowEditor for Effect {
    fn wrapper() -> Option<Self> {
        Some(Self::List([default()].into()))
    }
    fn show_children(&mut self, context: &Context, world: &mut World, ui: &mut Ui) {
        match self {
            Effect::Repeat(count, ef) => {
                count.show_node("count", context, world, ui);
                ef.show_node("effect", context, world, ui);
            }
            Effect::WithTarget(ex, ef) | Effect::WithOwner(ex, ef) | Effect::WithVar(_, ex, ef) => {
                ex.show_node("value", context, world, ui);
                ef.show_node("effect", context, world, ui);
            }
            Effect::AbilityStateAddVar(_, _, e) | Effect::Text(e) => {
                e.show_node("", context, world, ui)
            }

            Effect::Summon(_, e) => {
                if let Some(e) = e {
                    e.show_node("", context, world, ui);
                }
            }

            Effect::List(l) => {
                show_list_node(l, context, ui, world);
            }

            Effect::If(e, th, el) => {
                e.show_node("condition", context, world, ui);
                th.show_node("then", context, world, ui);
                el.show_node("else", context, world, ui);
            }
            Effect::StatusSetVar(target, _, _, value) | Effect::StateAddVar(_, target, value) => {
                target.show_node("target", context, world, ui);
                value.show_node("value", context, world, ui);
            }
            Effect::Noop
            | Effect::Damage
            | Effect::Kill
            | Effect::Heal
            | Effect::ChangeAllStatuses
            | Effect::ClearAllStatuses
            | Effect::StealAllStatuses
            | Effect::FullCopy
            | Effect::ChangeStatus(_)
            | Effect::ClearStatus(_)
            | Effect::StealStatus(_)
            | Effect::UseAbility(_, _)
            | Effect::Vfx(_) => {}
        }
    }
    fn show_content(&mut self, _: &Context, world: &mut World, ui: &mut Ui) {
        match self {
            Effect::ChangeStatus(status)
            | Effect::ClearStatus(status)
            | Effect::StealStatus(status) => {
                status_selector(status, ui);
            }
            Effect::UseAbility(ability, base) => {
                ability_selector(ability, ui);
                DragValue::new(base).range(0..=10).ui(ui);
            }
            Effect::AbilityStateAddVar(ability, var, _) => {
                ability_selector(ability, ui);
                var_selector(var, ui);
            }
            Effect::StatusSetVar(_, status, var, _) => {
                status_selector(status, ui);
                var_selector(var, ui);
            }
            Effect::Summon(summon, _) => {
                summon_selector(summon, ui);
            }
            Effect::StateAddVar(var, _, _) | Effect::WithVar(var, _, _) => {
                var_selector(var, ui);
            }
            Effect::Vfx(vfx) => {
                vfx_selector(vfx, ui);
            }
            Effect::List(l) => {
                if Button::click("+").ui(ui).clicked() {
                    l.push(default());
                }
            }

            Effect::WithTarget(_, _)
            | Effect::WithOwner(_, _)
            | Effect::Repeat(_, _)
            | Effect::If(_, _, _)
            | Effect::Text(_)
            | Effect::ChangeAllStatuses
            | Effect::ClearAllStatuses
            | Effect::StealAllStatuses
            | Effect::FullCopy
            | Effect::Noop
            | Effect::Damage
            | Effect::Kill
            | Effect::Heal => {}
        }
    }
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        match self {
            Effect::WithTarget(_, e)
            | Effect::WithOwner(_, e)
            | Effect::WithVar(_, _, e)
            | Effect::Repeat(_, e) => [e].into(),
            Effect::List(l) => l.iter_mut().collect_vec(),
            Effect::If(_, a, b) => [a, b].into(),

            Effect::Summon(_, _)
            | Effect::Noop
            | Effect::Damage
            | Effect::Kill
            | Effect::Heal
            | Effect::ChangeStatus(_)
            | Effect::ClearStatus(_)
            | Effect::StealStatus(_)
            | Effect::ChangeAllStatuses
            | Effect::ClearAllStatuses
            | Effect::StealAllStatuses
            | Effect::UseAbility(_, _)
            | Effect::AbilityStateAddVar(_, _, _)
            | Effect::Vfx(_)
            | Effect::StateAddVar(_, _, _)
            | Effect::StatusSetVar(_, _, _, _)
            | Effect::Text(_)
            | Effect::FullCopy => default(),
        }
    }
    fn get_variants() -> impl Iterator<Item = Self> {
        Self::iter()
    }
}
