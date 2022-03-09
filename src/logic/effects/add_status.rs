use super::*;

impl Game {
    pub fn process_add_status_effect(
        &mut self,
        QueuedEffect { target, effect, .. }: QueuedEffect<AddStatusEffect>,
    ) {
        let target = target
            .and_then(|id| self.units.get_mut(&id))
            .expect("Target not found");
        self.render
            .add_text(target.position, effect.status.name(), Color::BLUE);
        target.statuses.push(effect.status);
    }
}
