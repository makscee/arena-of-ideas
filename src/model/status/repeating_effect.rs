use super::*;

fn zero() -> R32 {
    R32::ZERO
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RepeatingEffectStatus {
    #[serde(default = "zero")]
    pub next_tick: Time,
    pub tick_time: Option<Time>,
    pub effect: Effect,
}

impl EffectContainer for RepeatingEffectStatus {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl StatusImpl for RepeatingEffectStatus {}
