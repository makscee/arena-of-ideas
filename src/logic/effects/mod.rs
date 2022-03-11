use super::*;

mod add_status;
mod aoe;
mod chain;
mod damage;
mod modifiers;
mod projectile;
mod spawn;
mod suicide;
mod time_bomb;

pub use add_status::*;
pub use aoe::*;
pub use chain::*;
pub use damage::*;
pub use modifiers::*;
pub use projectile::*;
pub use spawn::*;
pub use suicide::*;
pub use time_bomb::*;

pub struct QueuedEffect<T> {
    pub effect: T,
    pub caster: Option<Id>,
    pub target: Option<Id>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeighedEffect {
    pub weight: f32,
    #[serde(flatten)]
    pub effect: Effect,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "type", deny_unknown_fields)]
pub enum Effect {
    Noop,
    Projectile(Box<ProjectileEffect>),
    Damage(Box<DamageEffect>),
    AddStatus(Box<AddStatusEffect>),
    Spawn(Box<SpawnEffect>),
    AOE(Box<AoeEffect>),
    TimeBomb(Box<TimeBombEffect>),
    Suicide(Box<SuicideEffect>),
    Chain(Box<ChainEffect>),
    Random { choices: Vec<WeighedEffect> },
    List { effects: Vec<Effect> },
}

impl Default for Effect {
    fn default() -> Self {
        Self::Noop
    }
}

impl Logic<'_> {
    pub fn process_effects(&mut self) {
        while let Some(effect) = self.effects.pop() {
            let caster = effect.caster;
            let target = effect.target;
            match effect.effect {
                Effect::Noop => {}
                Effect::Damage(effect) => self.process_damage_effect(QueuedEffect {
                    effect: *effect,
                    caster,
                    target,
                }),
                Effect::Projectile(effect) => self.process_projectile_effect(QueuedEffect {
                    effect: *effect,
                    caster,
                    target,
                }),
                Effect::AddStatus(effect) => self.process_add_status_effect(QueuedEffect {
                    effect: *effect,
                    caster,
                    target,
                }),
                Effect::Suicide(effect) => self.process_suicide_effect(QueuedEffect {
                    effect: *effect,
                    caster,
                    target,
                }),
                Effect::Spawn(effect) => self.process_spawn_effect(QueuedEffect {
                    effect: *effect,
                    caster,
                    target,
                }),
                Effect::TimeBomb(effect) => self.process_time_bomb_effect(QueuedEffect {
                    effect: *effect,
                    caster,
                    target,
                }),
                Effect::AOE(effect) => self.process_aoe_effect(QueuedEffect {
                    effect: *effect,
                    caster,
                    target,
                }),
                Effect::Chain(effect) => self.process_chain_effect(QueuedEffect {
                    effect: *effect,
                    caster,
                    target,
                }),
                Effect::List { effects } => {
                    for effect in effects {
                        self.effects.push(QueuedEffect {
                            effect,
                            caster,
                            target,
                        });
                    }
                }
                Effect::Random { choices } => {
                    let effect = choices
                        .choose_weighted(&mut global_rng(), |choice| choice.weight)
                        .unwrap()
                        .effect
                        .clone();
                    self.effects.push(QueuedEffect {
                        effect,
                        caster,
                        target,
                    });
                }
            }
        }
    }
}
