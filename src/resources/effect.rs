use legion::EntityStore;

use super::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Effect {
    Damage {
        value: Option<Hp>,
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
        status: String,
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
        value: ExpressionInt,
    },
    SetTarget {
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
        color: Rgba<f32>,
    },
}

impl Effect {
    pub fn process(
        &self,
        context: Context,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> Result<(), Error> {
        match self {
            Effect::Damage { value } => {
                Event::BeforeIncomingDamage.send(&context, resources);
                let value = match value {
                    Some(v) => *v,
                    None => world
                        .entry_ref(context.owner)
                        .context("Filed to get Owner")?
                        .get_component::<AttackComponent>()
                        .context("Failed to get Attack component")?
                        .value()
                        .clone(),
                };
                let mut text = format!("-{}", value);
                let mut target = world
                    .entry(context.target)
                    .context("Failed to get Target")?;
                if target
                    .get_component::<FlagsComponent>()?
                    .has_flag(&Flag::DamageImmune)
                {
                    debug!("Damage Immune");
                    text = "Immune".to_string();
                } else {
                    let hp = target.get_component_mut::<HpComponent>()?;
                    hp.set_current(hp.current() - value, resources);
                    debug!(
                        "Entity#{:?} {} damage taken, new hp: {}",
                        context.target,
                        value,
                        hp.current()
                    )
                }
                resources.cassette.add_effect(VisualEffect::new(
                    1.0,
                    VisualEffectType::ShaderAnimation {
                        shader: resources
                            .options
                            .text
                            .clone()
                            .set_uniform("u_text", ShaderUniform::String((0, text)))
                            .set_uniform("u_pivot", ShaderUniform::Float(3.0))
                            .set_uniform("u_scale_over_t", ShaderUniform::Float(-0.9))
                            .set_uniform("u_position_over_t", ShaderUniform::Vec2(vec2(0.0, 5.0)))
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
                ));
                Event::AfterIncomingDamage.send(&context, resources);
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
            Effect::List { effects } => effects.iter().for_each(|effect| {
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
            Effect::RemoveStatus { status } => {
                resources
                    .status_pool
                    .active_statuses
                    .get_mut(&context.target)
                    .context("Tried to remove absent status")?
                    .remove(status);
            }
            Effect::SetVarInt { name, value } => {
                let value = value.calculate(&context, world, resources);
                world
                    .entry(context.target)
                    .context("Failed to get Target")?
                    .get_component_mut::<Context>()?
                    .vars
                    .insert(name.clone(), Var::Int(value));
            }
            Effect::AttachStatus { name } => {
                StatusPool::add_entity_status(&context.target, name, context.clone(), resources);
            }
            Effect::UseAbility { name, house } => {
                if !world
                    .entry(context.owner)
                    .context("Failed to get Owner")?
                    .get_component::<HouseComponent>()?
                    .houses
                    .contains_key(house)
                {
                    panic!(
                        "Tried to use {} while not being a member of the {:?}",
                        name, house
                    );
                }
                let ability = resources
                    .houses
                    .get(house)
                    .context("Failed to get House")?
                    .abilities
                    .get(name)
                    .context("Failed to get Ability")?;
                resources.action_queue.push_front(Action::new(
                    {
                        let mut context = context.clone();
                        context.vars.merge(&ability.vars, true);
                        context
                    },
                    ability.effect.clone(),
                ));
            }
            Effect::ChangeStat { stat, value } => {
                let value = value.calculate(&context, world, resources);
                let mut target = world.entry(context.target).unwrap();
                match stat {
                    StatType::Hp => target
                        .get_component_mut::<HpComponent>()
                        .unwrap()
                        .set_current(value, resources),
                    StatType::Attack => target
                        .get_component_mut::<AttackComponent>()
                        .unwrap()
                        .set_value(value, resources),
                }
            }
            Effect::SetTarget { entity, effect } => {
                resources.action_queue.push_front(Action::new(
                    Context {
                        target: entity.calculate(&context, world, resources),
                        ..context.clone()
                    },
                    effect.deref().clone(),
                ));
            }
            Effect::SetAbilityVarInt {
                house,
                ability: ability_name,
                var: var_name,
                value,
            } => {
                let value = value.calculate(&context, world, resources);
                debug!(
                    "Set ability {} var {:?} value {}",
                    ability_name, var_name, value
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
                let value = value.calculate(&context, world, resources);
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
                if condition.calculate(&context, world, resources) {
                    resources
                        .action_queue
                        .push_front(Action::new(context.clone(), then.deref().clone()));
                } else {
                    resources
                        .action_queue
                        .push_front(Action::new(context.clone(), r#else.deref().clone()));
                }
            }
            Effect::ShowText { text, color } => resources.cassette.add_effect(VisualEffect::new(
                1.5,
                VisualEffectType::ShaderAnimation {
                    shader: resources
                        .options
                        .text
                        .clone()
                        .set_uniform("u_text", ShaderUniform::String((0, text.clone())))
                        .set_uniform("u_outline_color", ShaderUniform::Color(*color))
                        .set_uniform("u_alpha_over_t", ShaderUniform::Float(-1.0))
                        .set_uniform("u_scale", ShaderUniform::Float(0.4))
                        .set_uniform("u_position_over_t", ShaderUniform::Vec2(vec2(0.0, 5.0)))
                        .set_uniform(
                            "u_position",
                            ShaderUniform::Vec2(
                                world
                                    .entry_ref(context.target)
                                    .context("Failed to get target")?
                                    .get_component::<PositionComponent>()
                                    .unwrap()
                                    .0,
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
            )),
        }
        Ok(())
    }
}
