use super::*;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Who {
    Caster,
    From,
    Target,
}

fn default_who() -> Who {
    Who::Target
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttachStatusEffect {
    #[serde(default = "default_who")]
    pub who: Who,
    pub status: Status,
}

impl EffectContainer for AttachStatusEffect {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for AttachStatusEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let status_name = &effect.status.name.clone();
        let target = context.get(effect.who);
        if let Some(target) = target.and_then(|id| logic.model.units.get_mut(&id)) {
            if let Some(render) = &mut logic.render {
                render.add_text(
                    target.position,
                    &format!("{:?}", effect.status.name),
                    Color::BLUE,
                );
            }

            target
                .all_statuses
                .push(effect.status.attach(Some(target.id), context.caster));

            let target = target.id;
            let target = logic.model.units.get(&target).unwrap();

            for (effect, vars) in target.all_statuses.iter().flat_map(|status| {
                status.trigger(|trigger| match trigger {
                    StatusTrigger::SelfDetect {
                        status_name: detect,
                    } => detect == status_name,
                    _ => false,
                })
            }) {
                logic.effects.push_front(QueuedEffect {
                    effect,
                    context: EffectContext {
                        caster: Some(target.id),
                        from: Some(target.id),
                        target: Some(target.id),
                        vars,
                    },
                })
            }

            for other in &logic.model.units {
                for (effect, vars) in other.all_statuses.iter().flat_map(|status| {
                    status.trigger(|trigger| match trigger {
                        StatusTrigger::Detect {
                            status_name: detect,
                            filter,
                        } => {
                            other.id != target.id
                                && detect == status_name
                                && filter.matches(target.faction, other.faction)
                        }
                        _ => false,
                    })
                }) {
                    logic.effects.push_front(QueuedEffect {
                        effect,
                        context: EffectContext {
                            caster: Some(other.id),
                            from: Some(other.id),
                            target: Some(target.id),
                            vars,
                        },
                    })
                }
            }
        }
    }
}
