use super::*;

impl Game {
    pub fn deal_damage(
        &mut self,
        mut attacker: Option<&mut Unit>,
        target: &mut Unit,
        effects: &[Effect],
    ) {
        for effect in effects {
            self.apply_effect(effect, attacker.as_deref_mut(), target);
        }
    }
}
