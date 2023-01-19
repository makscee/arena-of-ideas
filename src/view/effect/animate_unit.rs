use super::*;
use tween::*;

pub struct AnimateUnitVisualEffect {
    pub id: Id,
    pub from: Position,
    pub to: Position,
    pub duration: Time,
    pub tween: AnimateTween,
}

pub enum AnimateTween {
    Linear,
    QuartOut,
    QuartIn,
    QuartInOut,
}

impl AnimateUnitVisualEffect {
    pub fn new(id: Id, from: Position, to: Position, duration: Time, tween: AnimateTween) -> Self {
        Self {
            id,
            from,
            to,
            duration,
            tween,
        }
    }
}

impl VisualEffect for AnimateUnitVisualEffect {
    fn update(&self, model: &mut VisualNodeModel, t: Time) {
        let unit = model
            .units
            .get_mut(&self.id)
            .expect(&format!("Attempt to animate absent unit#{}", self.id));
        unit.position += vec2(
            self.get_value(self.from.x, self.to.x, t),
            self.get_value(self.from.y, self.to.y, t),
        );
    }

    fn get_duration(&self) -> Time {
        self.duration
    }
}

impl AnimateUnitVisualEffect {
    fn get_value(&self, from: f32, to: f32, t: Time) -> f32 {
        match self.tween {
            AnimateTween::Linear => Tweener::linear(from, to, self.get_duration()).move_to(t),
            AnimateTween::QuartOut => Tweener::quart_out(from, to, self.get_duration()).move_to(t),
            AnimateTween::QuartIn => Tweener::quart_in(from, to, self.get_duration()).move_to(t),
            AnimateTween::QuartInOut => {
                Tweener::quart_in_out(from, to, self.get_duration()).move_to(t)
            }
        }
    }
}
