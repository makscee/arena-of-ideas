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
    pub status: StatusConfig,
}

impl EffectContainer for AttachStatusEffect {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for AttachStatusEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let status_name = &effect.status.name;
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
            for other in &logic.model.units {
                for status in &other.all_statuses {
                    // TODO: reimplement
                    // if let StatusOld::Detect(status) = status {
                    //     if other.id == target.id {
                    //         continue;
                    //     }
                    //     if status.detect_type != status_name {
                    //         continue;
                    //     }
                    //     if !status.on.matches(target.faction, other.faction) {
                    //         continue;
                    //     }
                    //     logic.effects.push_front(QueuedEffect {
                    //         effect: status.effect.clone(),
                    //         context: EffectContext {
                    //             caster: Some(other.id),
                    //             from: Some(other.id),
                    //             target: Some(target.id),
                    //             vars: default(),
                    //         },
                    //     });
                    // }
                    // if let StatusOld::SelfDetect(status) = status {
                    //     if other.id != target.id {
                    //         continue;
                    //     }
                    //     if status.detect_type != status_name {
                    //         continue;
                    //     }
                    //     logic.effects.push_front(QueuedEffect {
                    //         effect: status.effect.clone(),
                    //         context: EffectContext {
                    //             caster: Some(other.id),
                    //             from: Some(other.id),
                    //             target: Some(target.id),
                    //             vars: default(),
                    //         },
                    //     });
                    // }
                }
            }
        }
    }
}
