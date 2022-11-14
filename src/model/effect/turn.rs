use super::*;
use crate::model::effect::position_tween::Tween;

const UNIT_STARTING_OFFSET: Vec2<f32> = vec2(0.5, 0.0);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TurnPhase {
    None,
    PreStrike,
    Strike,
    PostStrike,
}

impl TurnPhase {
    pub fn duration(&self) -> Time {
        match self {
            TurnPhase::None => Time::new(0.3),
            TurnPhase::PreStrike => Time::new(0.3),
            TurnPhase::Strike => Time::new(0.05),
            TurnPhase::PostStrike => Time::new(0.1),
        }
    }
    fn position(&self, unit: &Unit) -> Vec2<RealImpl<f32>> {
        let unit_faction_factor = match unit.faction {
            Faction::Player => r32(-1.0),
            Faction::Enemy => r32(1.0),
        };
        match self {
            TurnPhase::None => unit.position.to_world(),
            TurnPhase::PreStrike => {
                unit.position.to_world()
                    + UNIT_STARTING_OFFSET.map(|x| r32(x)) * unit_faction_factor
            }
            TurnPhase::Strike => vec2(unit_faction_factor * unit.render.radius, R32::ZERO),
            TurnPhase::PostStrike => unit.position.to_world(),
        }
    }
    pub fn tween(&self) -> Tween {
        match self {
            TurnPhase::None => Tween::Linear,
            TurnPhase::PreStrike => Tween::CircOut,
            TurnPhase::Strike => Tween::Linear,
            TurnPhase::PostStrike => Tween::Linear,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TurnEffect {
    pub phase: TurnPhase,
    pub player: Id,
    pub enemy: Id,
}

impl EffectContainer for TurnEffect {
    fn walk_effects_mut(&mut self, f: &mut dyn FnMut(&mut Effect)) {}
}

impl EffectImpl for TurnEffect {
    fn process(self: Box<Self>, context: EffectContext, logic: &mut logic::Logic) {
        let effect = *self;
        let units = vec![
            logic.model.get(effect.player, &context),
            logic.model.get(effect.enemy, &context),
        ];
        let prev_phase = match effect.phase {
            TurnPhase::None => None,
            TurnPhase::PreStrike => Some(TurnPhase::None),
            TurnPhase::Strike => Some(TurnPhase::PreStrike),
            TurnPhase::PostStrike => Some(TurnPhase::Strike),
        };
        for unit in units.iter() {
            let context = {
                let mut context = context.clone();
                context.queue_id = Some(format!("Turn#{}", unit.id));
                context.owner = unit.id;
                context.target = unit.id;
                context.creator = unit.id;
                context
            };
            logic.effects.push_back(
                // add position change animation
                context.clone(),
                Effect::PositionTween(Box::new(PositionTweenEffect {
                    target: unit.id,
                    start_position: prev_phase
                        .clone()
                        .and_then(|p| Some(prev_phase.clone().unwrap().position(unit))),
                    position: effect.phase.position(unit),
                    duration: effect.phase.duration(),
                    t: R32::ZERO,
                    tween: effect.phase.tween(),
                })),
            );
            match effect.phase {
                TurnPhase::PostStrike => logic.effects.push_front(
                    {
                        let mut context = context.clone();
                        context.owner = unit.id;
                        context.creator = unit.id;
                        context.target = units
                            .iter()
                            .find(|u| u.id != unit.id)
                            .expect("Target for action not found")
                            .id;
                        context
                    },
                    unit.action.clone(),
                ),
                _ => {}
            }
        }
    }
}
