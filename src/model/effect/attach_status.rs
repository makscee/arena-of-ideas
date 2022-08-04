use crate::model::status::StatusAction;

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
    pub status: StatusRef,
    #[serde(default)]
    pub vars: HashMap<VarName, R32>,
}

impl EffectContainer for AttachStatusEffect {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for AttachStatusEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let status_name = effect.status.name();
        let target = context.get(effect.who);
        if let Some(target) = target.and_then(|id| logic.model.units.get_mut(&id)) {
            // Check if unit is immune to status attachment
            if target
                .flags
                .iter()
                .any(|flag| matches!(flag, UnitStatFlag::AttachStatusImmune))
            {
                return;
            }

            let mut status = effect.status.get(&logic.model.statuses).clone().attach(
                Some(target.id),
                context.caster,
                &mut logic.model.next_id,
            );
            if !status.status.hidden {
                logic.model.render_model.add_text(
                    target.position,
                    &format!("+{}", status_name),
                    status.status.color,
                    crate::render::TextType::Status,
                );
            }

            status.vars.extend(effect.vars.into_iter());
            let attached_status_id = unit_attach_status(status, &mut target.all_statuses);

            let target = target.id;
            let target = logic.model.units.get(&target).unwrap();

            for (effect, vars, status_id, status_color) in
                target.all_statuses.iter().flat_map(|status| {
                    status.trigger(|trigger| match trigger {
                        StatusTrigger::SelfDetectAttach {
                            status_name: detect,
                            status_action,
                        } => detect == status_name && status_action == &StatusAction::Add,
                        _ => false,
                    })
                })
            {
                logic.effects.push_front(QueuedEffect {
                    effect,
                    context: EffectContext {
                        caster: context.caster,
                        from: Some(target.id),
                        target: Some(target.id),
                        vars,
                        status_id: Some(attached_status_id),
                        color: Some(status_color),
                    },
                })
            }

            for other in &logic.model.units {
                for (effect, vars, status_id, status_color) in
                    other.all_statuses.iter().flat_map(|status| {
                        status.trigger(|trigger| match trigger {
                            StatusTrigger::DetectAttach {
                                status_name: detect,
                                filter,
                                status_action,
                            } => {
                                other.id != target.id
                                    && detect == status_name
                                    && status_action == &StatusAction::Add
                                    && filter.matches(target.faction, other.faction)
                            }
                            _ => false,
                        })
                    })
                {
                    logic.effects.push_front(QueuedEffect {
                        effect,
                        context: EffectContext {
                            caster: context.caster,
                            from: Some(other.id),
                            target: Some(target.id),
                            vars,
                            status_id: Some(attached_status_id),
                            color: Some(status_color),
                        },
                    })
                }
            }
        }
    }
}
