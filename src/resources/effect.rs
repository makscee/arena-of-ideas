use legion::EntityStore;

use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Effect {
    Damage {
        value: Option<ExpressionInt>,
        then: Option<Box<EffectWrapped>>,
    },
    Kill {
        then: Option<Box<EffectWrapped>>,
    },
    Repeat {
        count: usize,
        effect: Box<EffectWrapped>,
    },
    List {
        effects: Vec<Box<EffectWrapped>>,
    },
    Debug {
        message: String,
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
    },
    RemoveStatus {
        name: String,
    },
    RemoveThisStatus,
    ChangeStatus {
        name: String,
        charges: ExpressionInt,
    },
    UseAbility {
        house: HouseName,
        name: String,
    },
    SetHealth {
        value: ExpressionInt,
    },
    SetAttack {
        value: ExpressionInt,
    },
    ChangeContext {
        owner: Option<ExpressionEntity>,
        parent: Option<ExpressionEntity>,
        effect: Box<EffectWrapped>,
    },
    TakeVar {
        var: VarName,
        new_name: Option<VarName>,
        entity: ExpressionEntity,
        effect: Box<EffectWrapped>,
    },
    If {
        condition: Condition,
        then: Box<EffectWrapped>,
        r#else: Box<EffectWrapped>,
    },
    ShowText {
        text: String,
        color: Option<Rgba<f32>>,
    },
    Aoe {
        factions: Vec<Faction>,
        effect: Box<EffectWrapped>,
    },
    Revive {
        slot: Option<usize>,
    },
    RemoveTrigger,
}

impl Effect {
    pub fn wrap(self) -> EffectWrapped {
        EffectWrapped {
            effect: self,
            target: default(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EffectWrapped {
    #[serde(flatten)]
    pub effect: Effect,
    pub target: Option<ExpressionEntity>,
}

const DAMAGE_TEXT_EFFECT_KEY: &str = "damage_text";
const DAMAGE_CURVE_EFFECT_KEY: &str = "damage_curve";
const MULTIPLE_DAMAGE_DELAY: Time = 0.2;

impl EffectWrapped {
    pub fn process(
        &self,
        context: Context,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> Result<(), Error> {
        let mut context = Context {
            target: self
                .target
                .as_ref()
                .and_then(|t| t.calculate(&context, world, resources).ok())
                .unwrap_or(context.target),
            ..context
        };
        match &self.effect {
            Effect::Damage { value, then } => {
                let mut value = match value {
                    Some(v) => v.calculate(&context, world, resources)? as usize,
                    None => {
                        world
                            .entry_ref(context.owner)
                            .context("Failed to get Owner")?
                            .get_component::<AttackComponent>()
                            .context("Failed to get Attack component")?
                            .value
                    }
                };
                context.vars.insert(VarName::Damage, Var::Int(value as i32));
                Event::BeforeIncomingDamage {
                    context: context.clone(),
                }
                .send(world, resources);
                context = Event::ModifyIncomingDamage { context }.calculate(world, resources);
                value = context.vars.get_int(&VarName::Damage).max(0) as usize;
                let text = format!("-{}", value);
                let mut target = world
                    .entry(context.target)
                    .context("Failed to get Target")?;

                let effect_key = format!("{}_{:?}", DAMAGE_TEXT_EFFECT_KEY, context.owner);
                let effect_delay =
                    resources.cassette.get_key_count(&effect_key) as f32 * MULTIPLE_DAMAGE_DELAY;
                VfxSystem::vfx_show_text(
                    resources,
                    &text,
                    resources.options.colors.damage_text,
                    target.get_component::<AreaComponent>().unwrap().position,
                    0,
                );
                if value > 0 {
                    let hp = target.get_component_mut::<HealthComponent>()?;
                    hp.damage += value as usize;
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
                        &format!("Entity#{:?} {} damage taken", context.target, value),
                        &LogContext::Effect,
                    );
                    if let Some(effect) = then {
                        resources
                            .action_queue
                            .push_front(Action::new(context.clone(), effect.deref().clone()));
                    }
                    Event::AfterDamageDealt {
                        context: context.clone(),
                    }
                    .send(world, resources);
                }
                Event::AfterIncomingDamage { context }.send(world, resources);
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
            Effect::SetVarInt { name, value } => {
                let value = value.calculate(&context, world, resources)?;
                world
                    .entry(context.target)
                    .context("Failed to get Target")?
                    .get_component_mut::<Context>()?
                    .vars
                    .insert(name.clone(), Var::Int(value));
            }
            Effect::ChangeStatus { name, .. }
            | Effect::AddStatus { name, .. }
            | Effect::RemoveStatus { name, .. } => {
                let charges = match &self.effect {
                    Effect::AddStatus { name: _ } => 1,
                    Effect::RemoveStatus { name: _ } => -1,
                    Effect::ChangeStatus { name: _, charges } => {
                        charges.calculate(&context, world, resources)?
                    }
                    _ => 0,
                };
                StatusPool::change_entity_status(context.target, &name, resources, charges);
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
                let owner_entry = world
                    .entry_ref(context.owner)
                    .context("Failed to get Owner")?;
                if owner_entry
                    .get_component::<HouseComponent>()?
                    .houses
                    .get(&house)
                    .is_none()
                {
                    panic!(
                        "Tried to use {} while not being a member of the {:?}",
                        name, house
                    );
                }
                let defaults = resources
                    .house_pool
                    .try_get_ability_vars(house, name)
                    .context("Failed to find ability")?;
                if let Some(overrides) =
                    TeamPool::try_get_team(Faction::from_entity(context.owner, world), resources)
                        .and_then(|x| x.ability_state.get_vars(house, name))
                {
                    context.vars.merge_mut(overrides, true);
                }
                context.vars.merge_mut(defaults, true);
                context.vars.insert(
                    VarName::Color,
                    Var::Color(resources.house_pool.get_color(house)),
                );
                resources.action_queue.push_front(Action::new(
                    context,
                    resources.house_pool.get_ability(house, name).effect.clone(),
                ));
            }
            Effect::SetHealth { value } => {
                let value = value.calculate(&context, world, resources)?;
                let mut target = world.entry(context.target).unwrap();
                target.get_component_mut::<HealthComponent>().unwrap().value = value;
            }
            Effect::SetAttack { value } => {
                let value = value.calculate(&context, world, resources)? as usize;
                let mut target = world.entry(context.target).unwrap();
                target.get_component_mut::<AttackComponent>().unwrap().value = value;
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

                let prev_value = ExpressionInt::AbilityVar {
                    house: *house,
                    ability: ability.clone(),
                    var: *var,
                }
                .calculate(&context, world, resources)?;
                TeamPool::set_ability_var_int(
                    house,
                    ability,
                    var,
                    prev_value + delta,
                    &Faction::from_entity(context.owner, world),
                    resources,
                );
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
                    resources, &text, color, position, 2,
                ))
            }
            Effect::ChangeContext {
                owner,
                parent,
                effect,
            } => {
                resources.action_queue.push_front(Action::new(
                    Context {
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
            Effect::Kill { then } => {
                let mut entry = world.entry_mut(context.target).unwrap();
                let health = entry.get_component_mut::<HealthComponent>().unwrap();
                health.damage = i32::MAX as usize;
                if UnitSystem::process_death(context.target, world, resources) {
                    if let Some(effect) = then {
                        resources
                            .action_queue
                            .push_front(Action::new(context, effect.deref().clone()));
                    }
                }
            }
            Effect::Revive { slot } => {
                let slot = slot.unwrap_or(1);
                let (mut corpse, faction) = resources
                    .unit_corpses
                    .remove(&context.target)
                    .context("Target is not a corpse")?;
                corpse.health = 1;
                corpse.unpack(world, resources, slot, faction);
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
                        .get(&var)
                        .clone(),
                    );
                    Context { vars, ..context }
                },
                effect.deref().clone(),
            )),
            Effect::RemoveTrigger => {
                if let Some(mut entry) = world.entry(context.target) {
                    entry.remove_component::<Trigger>();
                }
            }
        }
        Ok(())
    }
}
