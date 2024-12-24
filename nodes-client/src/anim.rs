use serde::{Deserialize, Serialize};

use super::*;

pub struct Animator<'w, 's> {
    targets: Vec<Entity>,
    context: Context<'w, 's>,
    duration: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum AnimAction {
    Translate(Expression),
    Target(Expression),
    Duration(Expression),
}

pub struct AnimChange {
    pub entity: Entity,
    pub duration: f32,
    pub timeframe: f32,
    pub vars: Vec<(VarName, VarValue)>,
}

impl AnimAction {
    pub fn apply(&self, a: &mut Animator) -> Result<Vec<AnimChange>, ExpressionError> {
        let mut changes = Vec::default();
        match self {
            AnimAction::Translate(x) => {
                let pos = x.get_vec2(&a.context)?;
                for target in a.targets.iter().copied() {
                    changes.push(AnimChange {
                        entity: target,
                        duration: 0.1,
                        timeframe: 0.1,
                        vars: [(VarName::position, pos.into())].into(),
                    });
                }
            }
            AnimAction::Target(x) => {
                a.targets.push(x.get_entity(&a.context)?);
            }
            AnimAction::Duration(x) => {
                a.duration = x.get_f32(&a.context)?;
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
        }
    }
}
