use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AddStatusEffect {
    pub status: Status,
}

impl Logic<'_> {
    pub fn process_add_status_effect(
        &mut self,
        QueuedEffect { target, effect, .. }: QueuedEffect<AddStatusEffect>,
    ) {
        let target = target
            .and_then(|id| self.model.units.get_mut(&id))
            .expect("Target not found");
        if let Some(render) = &mut self.render {
            render.add_text(target.position, effect.status.name(), Color::BLUE);
        }
        target.statuses.push(effect.status);
    }
}
