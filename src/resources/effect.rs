use legion::EntityStore;

use super::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Effect {
    Damage { value: Option<Hp> },
    Repeat { count: usize, effect: Box<Effect> },
    List { effects: Vec<Box<Effect>> },
    Debug { message: String },
    AddFlag { flag: Flag },
    RemoveFlag { flag: Flag },
    RemoveStatus { status: String },
    Noop,
    AddVarInt { name: VarName, value: i32 },
    AttachStatus { name: String },
    UseAbility { house: HouseName, name: String },
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
                Event::BeforeIncomingDamage.send(&context, resources)?;
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
                resources.cassette.add_effect(VisualEffect {
                    duration: 1.0,
                    r#type: VisualEffectType::ShaderAnimation {
                        shader: resources
                            .options
                            .text
                            .clone()
                            .set_uniform("u_text", ShaderUniform::String(text))
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
                    order: 0,
                });
                Event::AfterIncomingDamage.send(&context, resources)?;
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
                    .push_back(Action::new(context.clone(), effect.deref().clone()))
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
            Effect::AddVarInt { name, value } => {
                world
                    .entry(context.target)
                    .context("Failed to get Target")?
                    .get_component_mut::<Context>()?
                    .vars
                    .insert(name.clone(), Var::Int(*value));
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
                    .contains(house)
                {
                    panic!(
                        "Tried to use {} while not being a member of the {:?}",
                        name, house
                    );
                }
                resources.action_queue.push_back(Action {
                    context,
                    effect: resources
                        .houses
                        .get(house)
                        .expect(&format!("Failed to get House: {:?}", house))
                        .abilities
                        .get(name)
                        .unwrap()
                        .effect
                        .clone(),
                });
            }
        }
        Ok(())
    }
}
