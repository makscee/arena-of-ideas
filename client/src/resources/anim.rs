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
    pub fn apply(&self, context: &mut ClientContext) -> NodeResult<()> {
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

#[derive(BevyComponent)]
pub struct Vfx;

impl AnimAction {
    fn apply(&self, a: &mut Animator, ctx: &mut ClientContext) -> NodeResult<()> {
        match self {
            AnimAction::translate(x) => {
                let pos = x.get_vec2(ctx)?;
                let mut t = ctx.t()?;
                for target in a.targets.iter().copied() {
                    ctx.world_mut()?
                        .get_mut::<NodeState>(target)
                        .to_not_found()?
                        .insert(t, a.duration, VarName::position, pos.into());
                    t += a.timeframe;
                }
                *ctx.t_mut()? = t;
            }
            AnimAction::set_target(x) => {
                a.targets = [x.get_entity(ctx)?].into();
            }
            AnimAction::add_target(x) => {
                a.targets.push(x.get_entity(ctx)?);
            }
            AnimAction::duration(x) => {
                a.duration = x.get_f32(ctx)?;
            }
            AnimAction::timeframe(x) => {
                a.timeframe = x.get_f32(ctx)?;
                a.duration = a.duration.at_least(a.timeframe);
            }
            AnimAction::list(vec) => {
                for aa in vec {
                    aa.apply(a, ctx)?;
                }
            }
            AnimAction::spawn(material) => {
                let entity = ctx.world_mut()?.spawn_empty().id();
                NUnitRepresentation {
                    material: *material.clone(),
                    ..default()
                }
                .spawn(ctx, entity)?;
                ctx.world_mut()?.entity_mut(entity).insert(Vfx);

                let mut t = ctx.t()?;
                let vars_layers = ctx.get_vars_layers();
                let mut state = ctx.load_mut::<NodeState>(entity)?;
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
                *ctx.t_mut()? = t;
            }
            AnimAction::wait(expression) => {
                *ctx.t_mut()? += expression.get_f32(ctx)?;
            }
        };
        Ok(())
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
