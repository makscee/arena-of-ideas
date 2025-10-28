use egui::NumExt;
use serde::{Deserialize, Serialize};

use super::*;

pub struct Animator {
    targets: Vec<u64>,
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
    pub fn apply(&self, ctx: &mut ClientContext) -> NodeResult<()> {
        let a = &mut Animator::new();
        for action in &self.actions {
            action.apply(a, ctx).track()?;
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
                let pos = x.get_vec2(ctx).track()?;
                let mut t = ctx.t().to_not_found()?;
                for target in a.targets.iter().copied() {
                    let entity = target.entity(ctx)?;
                    ctx.world_mut()?
                        .get_mut::<NodeStateHistory>(entity)
                        .to_not_found()?
                        .insert(t, a.duration, VarName::position, pos.into());
                    t += a.timeframe;
                }
                ctx.battle_mut()?.duration = t;
            }
            AnimAction::set_target(x) => {
                a.targets = x.get_u64_list(ctx).track()?;
            }
            AnimAction::add_target(x) => {
                a.targets.push(x.get_u64(ctx).track()?);
            }
            AnimAction::duration(x) => {
                a.duration = x.get_f32(ctx).track()?;
            }
            AnimAction::timeframe(x) => {
                a.timeframe = x.get_f32(ctx).track()?;
                a.duration = a.duration.at_least(a.timeframe);
            }
            AnimAction::list(vec) => {
                for aa in vec {
                    aa.apply(a, ctx).track()?;
                }
            }
            AnimAction::spawn(material) => {
                let entity = ctx.world_mut()?.spawn_empty().id();
                let id = next_id();
                NUnitRepresentation::new(0, *material.clone())
                    .with_id(id)
                    .spawn(ctx, Some(entity))
                    .track()?;
                ctx.world_mut()?.entity_mut(entity).insert(Vfx);

                let mut t = ctx.t().to_not_found().track()?;
                let vars_layers = ctx.get_vars_layers();
                let entity = ctx.entity(id).track()?;
                let mut state = ctx
                    .world_mut()?
                    .get_mut::<NodeStateHistory>(entity)
                    .to_not_found()
                    .track()?;
                state.insert(0.0, 0.0, VarName::visible, false.into());
                state.insert(t, 0.0, VarName::visible, true.into());
                state.insert(t + a.duration, 0.0, VarName::visible, false.into());
                state.insert(t, 0.0, VarName::t, 0.0.into());
                state.insert(t + 0.0001, a.duration, VarName::t, 1.0.into());
                debug!("spawn {material:?} layers {vars_layers:?}");
                for (var, value) in vars_layers {
                    state.insert(0.0, 0.0, var, value);
                }
                a.targets = vec![id];
                t += a.timeframe;
                ctx.battle_mut()?.duration = t;
            }
            AnimAction::wait(expression) => {
                ctx.battle_mut()?.duration += expression.get_f32(ctx)?;
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
