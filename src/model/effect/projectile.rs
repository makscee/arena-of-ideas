use super::*;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ProjectileEffect {
    pub speed: Coord,
    pub effect: Effect,
    #[serde(rename = "render", default = "ProjectileEffect::default_render")]
    pub render_config: RenderConfig,
}

impl ProjectileEffect {
    pub fn default_render() -> RenderConfig {
        RenderConfig::Circle {
            color: Color::MAGENTA,
        }
    }
}

impl EffectContainer for ProjectileEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {
        self.effect.walk_mut(f);
    }
}

impl EffectImpl for ProjectileEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let target = context
            .target
            .and_then(|id| logic.model.units.get(&id))
            .expect("Target not found");
        let from = context
            .from
            .and_then(|id| logic.model.units.get(&id))
            .expect("From not found");
        if target.id == from.id {
            error!("Projectile target == from");
            return;
        }
        logic.model.projectiles.insert(Projectile {
            id: logic.model.next_id,
            caster: context.caster.expect("Projectile caster is undefined"),
            target: target.id,
            position: from.position + (target.position - from.position).normalize() * from.radius,
            speed: effect.speed,
            target_position: target.position,
            effect: effect.effect,
            render_config: effect.render_config,
            vars: context.vars.clone(),
        });
        logic.model.next_id += 1;
    }
}
