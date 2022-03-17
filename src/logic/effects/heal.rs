pub use super::*;

impl Logic<'_> {
    pub fn process_heal_effect(
        &mut self,
        QueuedEffect { effect, context }: QueuedEffect<HealEffect>,
    ) {
        let target_unit = context
            .target
            .and_then(|id| self.model.units.get_mut(&id))
            .expect("Target not found");

        let heal = target_unit.max_hp * effect.hp.relative + effect.hp.absolute;
        let heal = min(heal, target_unit.max_hp - target_unit.hp);
        target_unit.hp += heal;
    }
}
