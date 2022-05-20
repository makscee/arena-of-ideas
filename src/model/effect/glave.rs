use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct GlaveEffect {
    pub targets: usize,
    pub jump_distance: Coord,
    pub effect: ProjectileEffect,
    pub jump_modifier: Modifier,
}

impl EffectContainer for GlaveEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_effects_mut(f);
    }
}

impl EffectImpl for GlaveEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let mut projectile = effect.effect;
        if effect.targets >= 1 {
            let mut inner_effect = Effect::Noop(Box::new(NoopEffect {}));
            std::mem::swap(&mut projectile.effect, &mut inner_effect);
            let first_effect = inner_effect.clone();
            inner_effect.apply_modifier(&effect.jump_modifier);
            projectile.effect = Effect::List(Box::new(ListEffect {
                effects: vec![
                    first_effect,
                    Effect::ChangeTarget(Box::new(ChangeTargetEffect {
                        filter: TargetFilter::Enemies,
                        condition: Condition::InRange {
                            max_distance: effect.jump_distance,
                        },
                        effect: Effect::Glave(Box::new(GlaveEffect {
                            targets: effect.targets - 1,
                            jump_distance: effect.jump_distance,
                            effect: ProjectileEffect {
                                effect: inner_effect,
                                render_config: projectile.render_config.clone(),
                                ..projectile
                            },
                            jump_modifier: effect.jump_modifier,
                        })),
                    })),
                ],
            }));
            logic.effects.push_front(QueuedEffect {
                effect: Effect::Projectile(Box::new(projectile)),
                context,
            });
        }
    }
}
