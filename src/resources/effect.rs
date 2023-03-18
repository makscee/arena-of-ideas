use legion::EntityStore;

use super::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Effect {
    Damage {
        value: Option<ExpressionInt>,
        then: Option<Box<Effect>>,
    },
    Kill {
        target: Option<ExpressionEntity>,
        then: Option<Box<Effect>>,
    },
    Repeat {
        count: usize,
        effect: Box<Effect>,
    },
    List {
        effects: Vec<Box<Effect>>,
    },
    Debug {
        message: String,
    },
    AddFlag {
        flag: Flag,
    },
    RemoveFlag {
        flag: Flag,
    },
    Noop,
    SetVarInt {
        name: VarName,
        value: ExpressionInt,
    },
    ChangeAbilityVarInt {
        house: HouseName,
        ability: String,
        var: VarName,
        delta: ExpressionInt,
    },
    AddStatus {
        name: String,
        target: Option<ExpressionEntity>,
    },
    RemoveStatus {
        name: String,
        target: Option<ExpressionEntity>,
    },
    RemoveThisStatus,
    ChangeStatus {
        name: String,
        target: Option<ExpressionEntity>,
        charges: ExpressionInt,
    },
    UseAbility {
        house: HouseName,
        name: String,
    },
    SetCurrentHp {
        target: Option<ExpressionEntity>,
        value: ExpressionInt,
    },
    ChangeStat {
        stat: StatType,
        target: Option<ExpressionEntity>,
        delta: ExpressionInt,
    },
    ChangeAttack {
        target: Option<ExpressionEntity>,
        delta: ExpressionInt,
    },
    ChangeMaxHp {
        target: Option<ExpressionEntity>,
        delta: ExpressionInt,
    },
    ChangeStats {
        target: Option<ExpressionEntity>,
        delta: ExpressionInt,
    },
    ChangeContext {
        target: Option<ExpressionEntity>,
        owner: Option<ExpressionEntity>,
        parent: Option<ExpressionEntity>,
        effect: Box<Effect>,
    },
    TakeVar {
        var: VarName,
        new_name: Option<VarName>,
        entity: ExpressionEntity,
        effect: Box<Effect>,
    },
    If {
        condition: Condition,
        then: Box<Effect>,
        r#else: Box<Effect>,
    },
    ShowText {
        text: String,
        color: Option<Rgba<f32>>,
    },
    Aoe {
        factions: Vec<Faction>,
        effect: Box<Effect>,
    },
}

const DAMAGE_TEXT_EFFECT_KEY: &str = "damage_text";
const DAMAGE_CURVE_EFFECT_KEY: &str = "damage_curve";
const MULTIPLE_DAMAGE_DELAY: Time = 0.2;

impl Effect {
    pub fn process(
        &self,
        context: Context,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> Result<(), Error> {
        match self {
            Effect::Damage { value, then } => {
                let mut context = context.clone();
                let mut value = match value {
                    Some(v) => v.calculate(&context, world, resources)?,
                    None => {
                        world
                            .entry_ref(context.owner)
                            .context("Failed to get Owner")?
                            .get_component::<AttackComponent>()
                            .context("Failed to get Attack component")?
                            .value
                    }
                };
                if value == 0 {
                    resources
                        .logger
                        .log("Attempt to do zero damage, returning.", &LogContext::Effect);
                    return Ok(());
                }
                context.vars.insert(VarName::Damage, Var::Int(value));
                context = Event::ModifyIncomingDamage { context }
                    .send(resources, world)
                    .unwrap();
                value = context.vars.get_int(&VarName::Damage);
                Event::BeforeIncomingDamage {
                    context: context.clone(),
                }
                .send(resources, &world);
                let mut text = format!("-{}", value);
                let owner_position = world
                    .entry_ref(context.owner)
                    .context("Failed to get owner")?
                    .get_component::<AreaComponent>()?
                    .position;
                let mut target = world
                    .entry(context.target)
                    .context("Failed to get Target")?;
                let effect_key = format!("{}_{:?}", DAMAGE_TEXT_EFFECT_KEY, context.owner);
                let effect_delay =
                    resources.cassette.get_key_count(&effect_key) as f32 * MULTIPLE_DAMAGE_DELAY;
                if target
                    .get_component::<FlagsComponent>()?
                    .has_flag(&Flag::DamageImmune)
                {
                    resources.logger.log("Damage Immune", &LogContext::Effect);
                    text = "Immune".to_string();
                } else {
                    let hp = target.get_component_mut::<HpComponent>()?;
                    hp.current -= value;
                    resources.cassette.add_effect(VisualEffect::new_delayed(
                        1.0,
                        effect_delay,
                        VisualEffectType::EntityShaderAnimation {
                            entity: context.target,
                            from: hashmap! {
                                "u_damage_taken" => ShaderUniform::Float(1.0),
                            }
                            .into(),
                            to: hashmap! {
                                "u_damage_taken" => ShaderUniform::Float(0.0),
                            }
                            .into(),
                            easing: EasingType::Linear,
                        },
                        -1,
                    ));
                    resources.logger.log(
                        &format!(
                            "Entity#{:?} {} damage taken, new hp: {}",
                            context.target, value, hp.current
                        ),
                        &LogContext::Effect,
                    );
                    if let Some(effect) = then {
                        resources
                            .action_queue
                            .push_front(Action::new(context.clone(), effect.deref().clone()));
                    }
                }
                Event::AfterIncomingDamage { context }.send(resources, world);
            }
            Effect::Repeat { count, effect } => {
                for _ in 0..*count {
                    resources
                        .action_queue
                        .push_front(Action::new(context.clone(), effect.deref().clone()));
                }
            }
            Effect::Debug { message } => debug!("Debug effect: {}", message),
            Effect::Noop => {}
            Effect::List { effects } => effects.iter().rev().for_each(|effect| {
                resources
                    .action_queue
                    .push_front(Action::new(context.clone(), effect.deref().clone()))
            }),
            Effect::AddFlag { flag } => {
                world
                    .entry(context.target)
                    .context("Failed to get Target")?
                    .get_component_mut::<FlagsComponent>()?
                    .add_flag(flag.clone());
            }
            Effect::RemoveFlag { flag } => {
                world
                    .entry(context.target)
                    .context("Failed to get Target")?
                    .get_component_mut::<FlagsComponent>()?
                    .remove_flag(flag);
            }
            Effect::SetVarInt { name, value } => {
                let value = value.calculate(&context, world, resources)?;
                world
                    .entry(context.target)
                    .context("Failed to get Target")?
                    .get_component_mut::<Context>()?
                    .vars
                    .insert(name.clone(), Var::Int(value));
            }
            Effect::ChangeStatus { name, target, .. }
            | Effect::AddStatus { name, target, .. }
            | Effect::RemoveStatus { name, target, .. } => {
                let context = Context {
                    target: target
                        .as_ref()
                        .and_then(|t| t.calculate(&context, world, resources).ok())
                        .unwrap_or(context.target),
                    ..context.clone()
                };
                let charges = match self {
                    Effect::AddStatus { name: _, target: _ } => 1,
                    Effect::RemoveStatus { name: _, target: _ } => -1,
                    Effect::ChangeStatus {
                        name: _,
                        target: _,
                        charges,
                    } => charges.calculate(&context, world, resources)?,
                    _ => 0,
                };
                StatusPool::change_entity_status(context.target, name, resources, charges);
            }
            Effect::RemoveThisStatus => {
                StatusPool::change_entity_status(
                    context.target,
                    &context.vars.get_string(&VarName::StatusName),
                    resources,
                    -1,
                );
            }
            Effect::UseAbility { name, house } => {
                if world
                    .entry(context.owner)
                    .context("Failed to get Owner")?
                    .get_component::<HouseComponent>()?
                    .houses
                    .get(house)
                    .is_none()
                {
                    panic!(
                        "Tried to use {} while not being a member of the {:?}",
                        name, house
                    );
                }
                let house = resources.houses.get(house).context("Failed to get House")?;
                let ability = house.abilities.get(name).context("Failed to get Ability")?;
                resources.action_queue.push_front(Action::new(
                    {
                        let mut context = context.clone();
                        context.vars.merge(&ability.vars, true);
                        context.vars.insert(VarName::Color, Var::Color(house.color));
                        context
                    },
                    ability.effect.clone(),
                ));
            }
            Effect::SetCurrentHp { value, target } => {
                let value = value.calculate(&context, world, resources)?;
                let context = Context {
                    target: target
                        .as_ref()
                        .and_then(|t| t.calculate(&context, world, resources).ok())
                        .unwrap_or(context.target),
                    ..context.clone()
                };
                let mut target = world.entry(context.target).unwrap();
                target.get_component_mut::<HpComponent>().unwrap().current = value;
            }
            Effect::ChangeStat {
                stat,
                delta,
                target,
            } => {
                let delta = delta.calculate(&context, world, resources)?;
                let context = Context {
                    target: target
                        .as_ref()
                        .and_then(|t| t.calculate(&context, world, resources).ok())
                        .unwrap_or(context.target),
                    ..context.clone()
                };
                let mut target = world.entry(context.target).unwrap();
                match stat {
                    StatType::Hp => {
                        let mut hp = target.get_component_mut::<HpComponent>().unwrap();
                        hp.max += delta;
                        if hp.current > 0 {
                            hp.current = (hp.current + delta).max(1);
                        } else if delta > 0 {
                            hp.current += delta;
                        }
                    }
                    StatType::Attack => {
                        target.get_component_mut::<AttackComponent>().unwrap().value += delta;
                    }
                }
            }
            Effect::ChangeAttack { target, delta } => {
                return Self::process(
                    &Effect::ChangeStat {
                        stat: StatType::Attack,
                        target: target.clone(),
                        delta: delta.clone(),
                    },
                    context,
                    world,
                    resources,
                );
            }
            Effect::ChangeMaxHp { target, delta } => {
                return Self::process(
                    &Effect::ChangeStat {
                        stat: StatType::Hp,
                        target: target.clone(),
                        delta: delta.clone(),
                    },
                    context,
                    world,
                    resources,
                );
            }
            Effect::ChangeStats { target, delta } => {
                return Self::process(
                    &Effect::List {
                        effects: vec![
                            Box::new(Effect::ChangeStat {
                                stat: StatType::Hp,
                                target: target.clone(),
                                delta: delta.clone(),
                            }),
                            Box::new(Effect::ChangeStat {
                                stat: StatType::Attack,
                                target: target.clone(),
                                delta: delta.clone(),
                            }),
                        ],
                    },
                    context,
                    world,
                    resources,
                );
            }
            Effect::ChangeAbilityVarInt {
                house,
                ability,
                var,
                delta,
            } => {
                let delta = delta.calculate(&context, world, resources)?;

                resources.logger.log(
                    &format!("Set ability {} var {:?} delta {}", ability, var, delta),
                    &LogContext::Effect,
                );
                let vars = &mut resources
                    .houses
                    .get_mut(house)
                    .unwrap()
                    .abilities
                    .get_mut(ability)
                    .unwrap()
                    .vars;
                let value = vars.try_get_int(var).unwrap_or_default() + delta;
                vars.insert(*var, Var::Int(value));
            }
            Effect::If {
                condition,
                then,
                r#else,
            } => {
                if condition.calculate(&context, world, resources)? {
                    resources
                        .action_queue
                        .push_front(Action::new(context.clone(), then.deref().clone()));
                } else {
                    resources
                        .action_queue
                        .push_front(Action::new(context.clone(), r#else.deref().clone()));
                }
            }
            Effect::ShowText { text, color } => {
                let position = context.vars.get_vec2(&VarName::Position);
                let color = color
                    .or_else(|| {
                        context
                            .vars
                            .try_get_color(&VarName::Color)
                            .or_else(|| Some(context.vars.get_color(&VarName::HouseColor1)))
                    })
                    .unwrap();
                resources.cassette.add_effect(VfxSystem::vfx_show_text(
                    resources, text, color, position, 2,
                ))
            }
            Effect::ChangeContext {
                target,
                owner,
                parent,
                effect,
            } => {
                resources.action_queue.push_front(Action::new(
                    Context {
                        target: match target {
                            Some(entity) => entity.calculate(&context, world, resources)?,
                            None => context.target,
                        },
                        owner: match owner {
                            Some(entity) => entity.calculate(&context, world, resources)?,
                            None => context.target,
                        },
                        parent: match parent {
                            Some(entity) => Some(entity.calculate(&context, world, resources)?),
                            None => context.parent,
                        },
                        ..context.clone()
                    },
                    effect.deref().clone(),
                ));
            }
            Effect::Kill { target, then } => {
                let context = Context {
                    target: target
                        .as_ref()
                        .and_then(|t| t.calculate(&context, world, resources).ok())
                        .unwrap_or(context.target),
                    ..context.clone()
                };
                world
                    .entry(context.target)
                    .unwrap()
                    .get_component_mut::<HpComponent>()
                    .unwrap()
                    .current = 0;
                if UnitSystem::process_death(context.target, world, resources) {
                    if let Some(effect) = then {
                        resources
                            .action_queue
                            .push_front(Action::new(context, effect.deref().clone()));
                    }
                }
            }
            Effect::Aoe { factions, effect } => {
                UnitSystem::collect_factions(world, &HashSet::from_iter(factions.clone()))
                    .iter()
                    .for_each(|(entity, _)| {
                        resources.action_queue.push_front(Action::new(
                            Context {
                                target: *entity,
                                ..context.clone()
                            },
                            effect.deref().clone(),
                        ));
                    })
            }
            Effect::TakeVar {
                var,
                entity,
                new_name,
                effect,
            } => resources.action_queue.push_front(Action::new(
                {
                    let mut vars = context.vars.clone();
                    vars.insert(
                        new_name.unwrap_or(*var),
                        ContextSystem::get_context(
                            entity.calculate(&context, world, resources)?,
                            world,
                        )
                        .vars
                        .get(var)
                        .clone(),
                    );
                    Context { vars, ..context }
                },
                effect.deref().clone(),
            )),
        }
        Ok(())
    }
}
