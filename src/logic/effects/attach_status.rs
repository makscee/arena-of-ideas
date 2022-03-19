use super::*;

impl Logic<'_> {
    pub fn process_attach_status_effect(
        &mut self,
        QueuedEffect { effect, context }: QueuedEffect<AttachStatusEffect>,
    ) {
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
        }
    }
}
