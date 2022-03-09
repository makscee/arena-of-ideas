use super::*;

mod add_status;
mod aoe;
mod damage;
mod spawn;
mod suicide;
mod time_bomb;

pub use add_status::*;
pub use aoe::*;
pub use damage::*;
pub use spawn::*;
pub use suicide::*;
pub use time_bomb::*;

pub struct QueuedEffect<T> {
    pub effect: T,
    pub caster: Option<Id>,
    pub target: Option<Id>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type")]
pub enum Effect {
    Damage(DamageEffect),
    AddStatus(AddStatusEffect),
    Spawn(SpawnEffect),
    AOE(AoeEffect),
    TimeBomb(TimeBombEffect),
    Suicide(SuicideEffect),
}

impl Game {
    pub fn process_effects(&mut self) {
        while let Some(effect) = self.effects.pop() {
            let caster = effect.caster;
            let target = effect.target;
            match effect.effect {
                Effect::Damage(effect) => self.process_damage_effect(QueuedEffect {
                    effect,
                    caster,
                    target,
                }),
                Effect::AddStatus(effect) => self.process_add_status_effect(QueuedEffect {
                    effect,
                    caster,
                    target,
                }),
                Effect::Suicide(effect) => self.process_suicide_effect(QueuedEffect {
                    effect,
                    caster,
                    target,
                }),
                Effect::Spawn(effect) => self.process_spawn_effect(QueuedEffect {
                    effect,
                    caster,
                    target,
                }),
                Effect::TimeBomb(effect) => self.process_time_bomb_effect(QueuedEffect {
                    effect,
                    caster,
                    target,
                }),
                Effect::AOE(effect) => self.process_aoe_effect(QueuedEffect {
                    effect,
                    caster,
                    target,
                }),
            }
        }
    }
}
