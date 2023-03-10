use legion::EntityStore;

use super::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Effect {
    Damage {
        value: Option<ExpressionInt>,
    },
    Kill {
        target: ExpressionEntity,
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
    RemoveStatus {
        name: String,
    },
    Noop,
    SetVarInt {
        name: VarName,
        value: ExpressionInt,
    },
    SetAbilityVarInt {
        house: HouseName,
        ability: String,
        var: VarName,
        value: ExpressionInt,
    },
    SetStatusVarInt {
        status: String,
        var: VarName,
        value: ExpressionInt,
    },
    AttachStatus {
        name: String,
    },
    UseAbility {
        house: HouseName,
        name: String,
    },
    ChangeStat {
        stat: StatType,
        target: Option<ExpressionEntity>,
        value: ExpressionInt,
    },
    ChangeContext {
        target: Option<ExpressionEntity>,
        owner: Option<ExpressionEntity>,
        parent: Option<ExpressionEntity>,
        effect: Box<Effect>,
    },
    TakeVar {
        var: VarName,
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
            Effect::Damage { value } => {
                let mut context = context.clone();
                let mut value = match value {
                    Some(v) => v.calculate(&context, world, resources)?,
                    None => {
                        world
                            .entry_ref(context.owner)
                            .context("Filed to get Owner")?
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
                    .get_component::<PositionComponent>()?
                    .0;
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
                }
                resources.cassette.add_effect_by_key(
                    &effect_key,
                    VisualEffect::new_delayed(
                        1.0,
                        effect_delay,
                        VisualEffectType::ShaderAnimation {
                            shader: resources
                                .options
                                .shaders
                                .text
                                .clone()
                                .set_uniform("u_text", ShaderUniform::String((0, text)))
                                .set_uniform("u_pivot", ShaderUniform::Float(3.0))
                                .set_uniform("u_scale_over_t", ShaderUniform::Float(-0.9))
                                .set_uniform(
                                    "u_position_over_t",
                                    ShaderUniform::Vec2(vec2(0.0, 5.0)),
                                )
                                .set_uniform("u_gravity", ShaderUniform::Float(-5.0))
                                .set_uniform(
                                    "u_position",
                                    ShaderUniform::Vec2(
                                        target.get_component::<PositionComponent>().unwrap().0,
                                    ),
                                ),
                            from: hashmap! {
                                "u_time" => ShaderUniform::Float(0.0),
                            }
                            .into(),
                            to: hashmap! {
                                "u_time" => ShaderUniform::Float(1.0),
                            }
                            .into(),
                            easing: EasingType::Linear,
                        },
                        0,
                    ),
                );
                let effect_key = format!("{}_{:?}", DAMAGE_CURVE_EFFECT_KEY, context.owner);
                resources.cassette.add_effect_by_key(
                    &effect_key,
                    VisualEffect::new_delayed(
                        1.0,
                        effect_delay,
                        VisualEffectType::ShaderAnimation {
                            shader: resources
                                .options
                                .shaders
                                .curve
                                .clone()
                                .set_uniform("u_position", ShaderUniform::Vec2(owner_position))
                                .set_uniform(
                                    "u_target_position",
                                    ShaderUniform::Vec2(
                                        target.get_component::<PositionComponent>().unwrap().0,
                                    ),
                                ),
                            from: hashmap! {
                                "u_time" => ShaderUniform::Float(0.0),
                            }
                            .into(),
                            to: hashmap! {
                                "u_time" => ShaderUniform::Float(1.0),
                            }
                            .into(),
                            easing: EasingType::Linear,
                        },
                        0,
                    ),
                );
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
            Effect::RemoveStatus { name: status } => {
                resources
                    .status_pool
                    .active_statuses
                    .get_mut(&context.target)
                    .context("Tried to remove absent status")?
                    .remove(status);
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
            Effect::AttachStatus { name } => {
                StatusPool::add_entity_status(context.target, name, context, resources);
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
            Effect::ChangeStat {
                stat,
                value,
                target,
            } => {
                let value = value.calculate(&context, world, resources)?;
                let target = target
                    .as_ref()
                    .and_then(|target| target.calculate(&context, world, resources).ok())
                    .unwrap_or(context.target);
                let mut target = world.entry(target).unwrap();
                match stat {
                    StatType::Hp => {
                        target.get_component_mut::<HpComponent>().unwrap().current = value
                    }
                    StatType::Attack => {
                        target.get_component_mut::<AttackComponent>().unwrap().value = value
                    }
                }
            }
            Effect::SetAbilityVarInt {
                house,
                ability: ability_name,
                var: var_name,
                value,
            } => {
                let value = value.calculate(&context, world, resources)?;

                resources.logger.log(
                    &format!(
                        "Set ability {} var {:?} value {}",
                        ability_name, var_name, value
                    ),
                    &LogContext::Effect,
                );
                resources
                    .houses
                    .get_mut(house)
                    .context(format!("Failed to get {:?}", house))?
                    .abilities
                    .get_mut(ability_name)
                    .context(format!(
                        "Failed to get Ability {} from {:?}",
                        ability_name, house
                    ))?
                    .vars
                    .insert(*var_name, Var::Int(value));
            }
            Effect::SetStatusVarInt { status, var, value } => {
                let value = value.calculate(&context, world, resources)?;
                resources
                    .status_pool
                    .active_statuses
                    .get_mut(&context.target)
                    .context(format!(
                        "Failed to get target#{:?} statuses",
                        context.target
                    ))?
                    .get_mut(status)
                    .context(format!(
                        "Failed to find status {} on {:?}",
                        status, context.target
                    ))?
                    .vars
                    .insert(*var, Var::Int(value));
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
                    target: target.calculate(&context, world, resources)?,
                    ..context.clone()
                };
                if UnitSystem::kill(context.target, world, resources) {
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
                effect,
            } => resources.action_queue.push_front(Action::new(
                {
                    let mut vars = context.vars.clone();
                    vars.insert(
                        *var,
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
