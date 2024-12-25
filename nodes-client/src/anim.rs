use egui::NumExt;
use serde::{Deserialize, Serialize};

use super::*;

pub struct Animator<'w, 's> {
    targets: Vec<Entity>,
    context: Context<'w, 's>,
    duration: f32,
    timeframe: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Anim {
    actions: Vec<AnimAction>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AnimAction {
    Translate(Expression),
    SetTarget(Expression),
    AddTarget(Expression),
    Duration(Expression),
    Timeframe(Expression),
}

#[derive(Debug)]
pub struct AnimChange {
    pub entity: Entity,
    pub duration: f32,
    pub timeframe: f32,
    pub vars: Vec<(VarName, VarValue)>,
}

impl Anim {
    pub fn new(actions: Vec<AnimAction>) -> Self {
        Self { actions }
    }
    pub fn get_changes(&self, context: Context) -> Result<Vec<AnimChange>, ExpressionError> {
        let mut a = Animator::new(context);
        let mut changes = Vec::default();
        for action in &self.actions {
            changes.extend(action.apply(&mut a)?);
        }
        Ok(changes)
    }
}

impl AnimChange {
    pub fn apply(self, t: &mut f32, world: &mut World) {
        let AnimChange {
            entity,
            duration,
            timeframe,
            vars,
        } = self;
        for (var, value) in vars {
            NodeState::from_world_mut(entity, world).unwrap().insert(
                *t,
                duration,
                var,
                value,
                NodeKind::None,
            );
            *t += timeframe;
        }
    }
}

impl AnimAction {
    fn apply(&self, a: &mut Animator) -> Result<Vec<AnimChange>, ExpressionError> {
        let mut changes = Vec::default();
        match self {
            AnimAction::Translate(x) => {
                let pos = x.get_vec2(&a.context)?;
                for target in a.targets.iter().copied() {
                    changes.push(AnimChange {
                        entity: target,
                        duration: a.duration,
                        timeframe: a.timeframe,
                        vars: [(VarName::position, pos.into())].into(),
                    });
                }
            }
            AnimAction::SetTarget(x) => {
                a.targets = [x.get_entity(&a.context)?].into();
            }
            AnimAction::AddTarget(x) => {
                a.targets.push(x.get_entity(&a.context)?);
            }
            AnimAction::Duration(x) => {
                a.duration = x.get_f32(&a.context)?;
            }
            AnimAction::Timeframe(x) => {
                a.timeframe = x.get_f32(&a.context)?;
                a.duration = a.duration.at_least(a.timeframe);
            }
        };
        Ok(changes)
    }
}

impl<'w, 's> Animator<'w, 's> {
    pub fn new(context: Context<'w, 's>) -> Self {
        Self {
            targets: Vec::new(),
            context,
            duration: 0.0,
            timeframe: 0.0,
        }
    }
}
