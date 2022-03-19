use super::*;

impl Logic<'_> {
    pub fn process_attach_status_effect(
        &mut self,
        QueuedEffect { effect, context }: QueuedEffect<AttachStatusEffect>,
    ) {
        let status_type = effect.status.status.r#type();
        let target = context.get(effect.who);
        if let Some(target) = target.and_then(|id| self.model.units.get_mut(&id)) {
            if let Some(render) = &mut self.render {
                render.add_text(
                    target.position,
                    &format!("{:?}", effect.status.status.r#type()),
                    Color::BLUE,
                );
            }
            target.attached_statuses.push(effect.status);

            let target = target.id;
            let target = self.model.units.get(&target).unwrap();
            for other in &self.model.units {
                for status in &other.attached_statuses {
                    if let Status::DetectAttachedStatus {
                        on,
                        status,
                        ref effect,
                    } = status.status
                    {
                        if other.id == target.id {
                            continue;
                        }
                        if status != status_type {
                            continue;
                        }
                        if !on.matches(target.faction, other.faction) {
                            continue;
                        }
                        self.effects.push_back(QueuedEffect {
                            effect: effect.clone(),
                            context: EffectContext {
                                caster: Some(other.id),
                                from: Some(other.id),
                                target: Some(target.id),
                            },
                        });
                    }
                }
            }
        }
    }
}
