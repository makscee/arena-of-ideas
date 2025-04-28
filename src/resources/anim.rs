use egui::NumExt;
use serde::{Deserialize, Serialize};

use super::*;

pub struct Animator {
    targets: Vec<Entity>,
    duration: f32,
    timeframe: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct Anim {
    actions: Vec<Box<AnimAction>>,
}

#[allow(non_camel_case_types)]
#[derive(Serialize, Deserialize, Clone, Debug, AsRefStr, EnumIter, PartialEq)]
pub enum AnimAction {
    translate(Box<Expression>),
    set_target(Box<Expression>),
    add_target(Box<Expression>),
    duration(Box<Expression>),
    timeframe(Box<Expression>),
    wait(Box<Expression>),
    spawn(Box<Material>),
    list(Vec<Box<Self>>),
}

impl Anim {
    pub fn new(actions: Vec<AnimAction>) -> Self {
        Self {
            actions: actions.into_iter().map(|a| Box::new(a)).collect(),
        }
    }
    pub fn apply(&self, context: &mut Context) -> Result<(), ExpressionError> {
        let a = &mut Animator::new();
        for action in &self.actions {
            action.apply(a, context)?;
        }
        Ok(())
    }
    pub fn push(&mut self, action: AnimAction) -> &mut Self {
        self.actions.push(Box::new(action));
        self
    }
}

impl AnimAction {
    fn apply(&self, a: &mut Animator, context: &mut Context) -> Result<f32, ExpressionError> {
        let mut end_t = 0.0;
        match self {
            AnimAction::translate(x) => {
                let pos = x.get_vec2(context)?;
                let mut t = context.t()?;
                for target in a.targets.iter().copied() {
                    context.get_mut::<NodeState>(target)?.insert(
                        t,
                        a.duration,
                        VarName::position,
                        pos.into(),
                    );
                    t += a.timeframe;
                }
                *context.t_mut()? = t;
            }
            AnimAction::set_target(x) => {
                a.targets = [x.get_entity(&context)?].into();
            }
            AnimAction::add_target(x) => {
                a.targets.push(x.get_entity(&context)?);
            }
            AnimAction::duration(x) => {
                a.duration = x.get_f32(&context)?;
            }
            AnimAction::timeframe(x) => {
                a.timeframe = x.get_f32(&context)?;
                a.duration = a.duration.at_least(a.timeframe);
            }
            AnimAction::list(vec) => {
                for aa in vec {
                    aa.apply(a, context)?;
                }
            }
            AnimAction::spawn(material) => {
                let entity = context.world_mut()?.spawn_empty().id();
                NRepresentation {
                    material: *material.clone(),
                    ..default()
                }
                .unpack_entity(context, entity)?;

                let mut t = context.t()?;
                let vars_layers = context.get_vars_layers();
                let mut state = context.get_mut::<NodeState>(entity)?;
                state.insert(0.0, 0.0, VarName::visible, false.into());
                state.insert(t, 0.0, VarName::visible, true.into());
                state.insert(t + a.duration, 0.0, VarName::visible, false.into());
                state.insert(t, 0.0, VarName::t, 0.0.into());
                state.insert(t + 0.0001, a.duration, VarName::t, 1.0.into());
                for (var, value) in vars_layers {
                    state.insert(0.0, 0.0, var, value);
                }
                a.targets = vec![entity];
                t += a.timeframe;
                *context.t_mut()? = t;
            }
            AnimAction::wait(expression) => {
                *context.t_mut()? += expression.get_f32(context)?;
            }
        };
        Ok(end_t)
    }
}

impl Animator {
    pub fn new() -> Self {
        Self {
            targets: Vec::new(),
            duration: 1.0,
            timeframe: 0.0,
        }
    }
}

impl Default for AnimAction {
    fn default() -> Self {
        Self::translate(Box::new(Expression::vec2(0.0, 0.0)))
    }
}
impl Inject for AnimAction {
    fn move_inner(&mut self, source: &mut Self) {
        <Self as Injector<Self>>::inject_inner(self, source);
        <Self as Injector<Expression>>::inject_inner(self, source);
    }
}
impl Injector<Self> for AnimAction {
    fn get_inner_mut(&mut self) -> Vec<&mut Self> {
        match self {
            AnimAction::translate(..)
            | AnimAction::set_target(..)
            | AnimAction::add_target(..)
            | AnimAction::duration(..)
            | AnimAction::timeframe(..)
            | AnimAction::wait(..)
            | AnimAction::spawn(..) => default(),
            AnimAction::list(vec) => vec.into_iter().map(|v| v.as_mut()).collect_vec(),
        }
    }
    fn get_inner(&self) -> Vec<&Self> {
        match self {
            AnimAction::translate(..)
            | AnimAction::set_target(..)
            | AnimAction::add_target(..)
            | AnimAction::duration(..)
            | AnimAction::timeframe(..)
            | AnimAction::wait(..)
            | AnimAction::spawn(..) => default(),
            AnimAction::list(vec) => vec.into_iter().map(|v| v.as_ref()).collect_vec(),
        }
    }
}
impl Injector<Expression> for AnimAction {
    fn get_inner_mut(&mut self) -> Vec<&mut Expression> {
        match self {
            AnimAction::translate(x)
            | AnimAction::set_target(x)
            | AnimAction::add_target(x)
            | AnimAction::duration(x)
            | AnimAction::wait(x)
            | AnimAction::timeframe(x) => [x.as_mut()].into(),
            AnimAction::list(..) | AnimAction::spawn(..) => default(),
        }
    }
    fn get_inner(&self) -> Vec<&Expression> {
        match self {
            AnimAction::translate(x)
            | AnimAction::set_target(x)
            | AnimAction::add_target(x)
            | AnimAction::duration(x)
            | AnimAction::wait(x)
            | AnimAction::timeframe(x) => [x.as_ref()].into(),
            AnimAction::list(..) | AnimAction::spawn(..) => default(),
        }
    }
}
impl ToCstr for AnimAction {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr_c(PURPLE)
    }
}
impl ToCstr for Anim {
    fn cstr(&self) -> Cstr {
        self.actions.iter().map(|a| a.cstr()).join(" ")
    }
}
