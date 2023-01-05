use super::*;
use tween::*;

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum Tween {
    Linear,
    CircIn,
    CircOut,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct PositionTweenEffect {
    pub target: Id,
    pub start_position: Option<Vec2<RealImpl<f32>>>,
    pub position: Vec2<RealImpl<f32>>,
    pub duration: Time,
    pub t: Time,
    pub tween: Tween,
}

impl EffectContainer for PositionTweenEffect {
    fn walk_effects_mut(&mut self, _f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for PositionTweenEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let mut effect = *self.clone();
        let mut target = logic.model.get_mut(effect.target);
        if effect.start_position.is_none() {
            effect.start_position = Some(target.position.to_world());
        }
        effect.t += logic.delta_time;
        let start_pos = effect.start_position.unwrap();
        let tween_t = r32(match effect.tween {
            Tween::Linear => {
                Tweener::linear(0.0, 1.0, effect.duration.as_f32()).move_to(effect.t.as_f32())
            }
            Tween::CircIn => {
                Tweener::circ_in(0.0, 1.0, effect.duration.as_f32()).move_to(effect.t.as_f32())
            }
            Tween::CircOut => {
                Tweener::circ_out(0.0, 1.0, effect.duration.as_f32()).move_to(effect.t.as_f32())
            }
        });
        target.render.render_position = start_pos + (effect.position - start_pos) * tween_t;
        if effect.t < effect.duration {
            logic
                .effects
                .add_delay_by_id(context.get_q_id(), logic.delta_time.as_f32());
            logic
                .effects
                .push_back(context.clone(), Effect::PositionTween(Box::new(effect)));
        }
    }
}
