use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, AsRefStr, EnumIter)]
pub enum Effect {
    #[default]
    Noop,
    Damage,
    ChangeStatus(String),
    StealStatus(String),
    StealAllStatuses,
    UseAbility(String, i32),
    Summon(String, Option<Box<Effect>>),
    WithTarget(Expression, Box<Effect>),
    WithVar(VarName, Expression, Box<Effect>),
    List(Vec<Effect>),
    Repeat(Expression, Box<Effect>),
    If(Expression, Box<Effect>, Box<Effect>),
    Vfx(String),
}

impl Effect {
    pub fn invoke(&self, context: &mut Context, world: &mut World) -> Result<()> {
        debug!("Processing {:?}\n{:?}", self, context);
        let owner = context.owner();
        match self {
            Effect::Noop => {}
            Effect::Damage => {
                let target = context.get_target()?;
                let value = context
                    .get_var(VarName::Damage, world)
                    .unwrap_or(context.get_var(VarName::Pwr, world)?)
                    .get_int()?;
                if value > 0 {
                    debug!("deal {value} dmg to {target:?}");
                    let mut state = VarState::get_mut(target, world);
                    state.change_int(VarName::Dmg, value);
                    state.set_value(VarName::LastAttacker, owner.into());
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
                    TextColumnPlugin::add(
                        target,
                        format!("-{value}").cstr_cs(RED, CstrStyle::Bold),
                        world,
                    );
                    Vfx::get("pain", world).set_parent(target).unpack(world)?;
                }
                Vfx::get("damage", world)
                    .attach_context(context, world)
                    .unpack(world)?;
            }
            Effect::ChangeStatus(name) => {
                let delta = context.get_charges(world).unwrap_or(1);
                Status::change_charges(&name, context.get_target()?, delta, world);
            }
            Effect::StealStatus(name) => {
                let target = context.get_target()?;
                let charges = context.get_charges(world).unwrap_or(1);
                if charges <= 0 {
                    return Err(anyhow!("Can't steal nonpositive charges amount"));
                }
                let c = Status::get_charges(name, target, world)?;
                let delta = c.min(charges);
                Status::change_charges(name, target, -delta, world);
                Status::change_charges(name, owner, delta, world);
            }
            Effect::StealAllStatuses => {
                let target = context.get_target()?;
                for (s, c) in VarState::get(target, world).all_statuses_at(gt().insert_head()) {
                    if c > 0 {
                        ActionPlugin::action_push_front(
                            Effect::StealStatus(s),
                            context.clone(),
                            world,
                        );
                    }
                }
            }
            Effect::UseAbility(name, base) => {
                let ability = GameAssets::get(world)
                    .abilities
                    .get(name)
                    .with_context(|| format!("Ability not found {name}"))
                    .unwrap();
                let charges = context
                    .get_var(VarName::Level, world)
                    .map(|v| v.get_int().unwrap())
                    .unwrap_or(1)
                    + *base;
                let caster = owner;
                let context = context
                    .clone()
                    .inject_ability_state(name, world)?
                    .set_var(VarName::Charges, VarValue::Int(charges))
                    .set_caster(caster)
                    .take();
                ActionPlugin::action_push_front(ability.effect.clone(), context, world);
                TextColumnPlugin::add(
                    caster,
                    "use "
                        .cstr()
                        .push(name.cstr_cs(name_color(name), CstrStyle::Bold))
                        .take(),
                    world,
                );
            }
            Effect::Summon(name, then) => {
                let unit = GameAssets::get(world)
                    .summons
                    .get(name)
                    .with_context(|| format!("Summon {name} not found"))
                    .unwrap()
                    .clone();
                let faction = context.get_faction(world)?;
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
                    .sorted_by_key(|e| -VarState::get(*e, world).get_int(VarName::Slot).unwrap())
                    .collect_vec();
                let delay = 0.2;
                for target in targets {
                    let context = context.set_target(target).clone();
                    ActionPlugin::action_push_front_with_delay(
                        effect.deref().clone(),
                        context,
                        delay,
                        world,
                    );
                }
            }
            Effect::WithVar(var, value, effect) => {
                let context = context
                    .set_var(*var, value.get_value(context, world)?)
                    .clone();
                ActionPlugin::action_push_front(effect.deref().clone(), context, world);
            }
            Effect::List(list) => {
                for effect in list.into_iter().rev() {
                    ActionPlugin::action_push_front(effect.clone(), context.clone(), world);
                }
            }
            Effect::Repeat(count, effect) => {
                let count = count.get_int(context, world)?;
                for _ in 0..count {
                    ActionPlugin::action_push_front(effect.deref().clone(), context.clone(), world);
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
                Vfx::get(name, world)
                    .attach_context(context, world)
                    .unpack(world)?;
            }
        }
        Ok(())
    }
}

impl ToCstr for Effect {
    fn cstr(&self) -> Cstr {
        match self {
            Effect::UseAbility(name, base) => {
                let name_base = if *base > 0 {
                    format!("{name} ({})", *base + 1)
                } else {
                    name.clone()
                };
                format!("use ability ")
                    .cstr()
                    .push(format!("{name_base}").cstr_cs(name_color(&name), CstrStyle::Bold))
                    .take()
            }
            _ => self.as_ref().cstr(),
        }
    }
}
