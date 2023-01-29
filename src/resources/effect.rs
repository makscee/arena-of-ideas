use super::*;

#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Effect {
    Damage { value: Hp },
    Repeat { count: usize, effect: Box<Effect> },
    List { effects: Vec<Box<Effect>> },
    Debug { message: String },
    AddFlag { flag: Flag },
    RemoveFlag { flag: Flag },
    RemoveStatus { status: String },
    Noop,
    AddVarInt { name: VarName, value: i32 },
}

impl Effect {
    pub fn process(
        &self,
        mut context: Context,
        world: &mut legion::World,
        resources: &mut Resources,
    ) -> Result<Context, Error> {
        match self {
            Effect::Damage { value } => {
                Event::BeforeIncomingDamage.send(&context, resources)?;
                let mut target = world
                    .entry(context.target)
                    .context("Failed to get Target")?;
                if target
                    .get_component::<FlagsComponent>()?
                    .has_flag(&Flag::DamageImmune)
                {
                    debug!("Damage Immune");
                } else {
                    let target_context = target.get_component::<Context>()?;
                    context.vars = target_context.vars.extend(&context.vars);
                    let new_hp = target
                        .get_component_mut::<HpComponent>()?
                        .current
                        .change(&mut context.vars, -value)?;
                    debug!(
                        "Target#{:?} took {} damage, new hp: {}",
                        context.target, value, new_hp
                    );
                }
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
                    .statuses
                    .active_statuses
                    .get_mut(&context.target)
                    .context("Tried to remove absent status")?
                    .remove(status);
            }
            Effect::AddVarInt { name, value } => {
                context.vars.set_int(name.clone(), *value);
            }
        }
        Ok(context)
    }
}
