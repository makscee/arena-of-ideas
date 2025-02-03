use egui::NumExt;
use serde::{Deserialize, Serialize};

use super::*;

pub struct Animator<'w, 's> {
    targets: Vec<Entity>,
    context: Context<'w, 's>,
    duration: f32,
    timeframe: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Anim {
    actions: Vec<Box<AnimAction>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, AsRefStr, EnumIter, PartialEq)]
pub enum AnimAction {
    Translate(Box<Expression>),
    SetTarget(Box<Expression>),
    AddTarget(Box<Expression>),
    Duration(Box<Expression>),
    Timeframe(Box<Expression>),
    Wait(Box<Expression>),
    Spawn(Box<Material>),
    List(Vec<Box<Self>>),
}

impl Anim {
    pub fn new(actions: Vec<AnimAction>) -> Self {
        Self {
            actions: actions.into_iter().map(|a| Box::new(a)).collect(),
        }
    }
    pub fn apply(
        &self,
        t: &mut f32,
        context: Context,
        world: &mut World,
    ) -> Result<f32, ExpressionError> {
        let a = &mut Animator::new(context);
        let mut end_t = *t;
        for action in &self.actions {
            end_t = end_t.max(action.apply(t, a, world)?);
        }
        Ok(end_t)
    }
    pub fn push(&mut self, action: AnimAction) -> &mut Self {
        self.actions.push(Box::new(action));
        self
    }
}

impl AnimAction {
    fn apply(
        &self,
        t: &mut f32,
        a: &mut Animator,
        world: &mut World,
    ) -> Result<f32, ExpressionError> {
        let mut end_t = 0.0;
        match self {
            AnimAction::Translate(x) => {
                let pos = x.get_vec2(&a.context.with_world(world))?;
                for target in a.targets.iter().copied() {
                    NodeState::from_world_mut(target, world).unwrap().insert(
                        *t,
                        a.duration,
                        VarName::position,
                        pos.into(),
                        NodeKind::None,
                    );
                    end_t = *t + a.duration;
                    *t += a.timeframe;
                }
            }
            AnimAction::SetTarget(x) => {
                a.targets = [x.get_entity(&a.context.with_world(world))?].into();
            }
            AnimAction::AddTarget(x) => {
                a.targets.push(x.get_entity(&a.context.with_world(world))?);
            }
            AnimAction::Duration(x) => {
                a.duration = x.get_f32(&a.context.with_world(world))?;
            }
            AnimAction::Timeframe(x) => {
                a.timeframe = x.get_f32(&a.context.with_world(world))?;
                a.duration = a.duration.at_least(a.timeframe);
            }
            AnimAction::List(vec) => {
                for aa in vec {
                    end_t = end_t.max(aa.apply(t, a, world)?);
                }
            }
            AnimAction::Spawn(material) => {
                let entity = world.spawn_empty().id();
                Representation {
                    material: *material.clone(),
                    children: default(),
                    entity: None,
                }
                .unpack(entity, &mut world.commands());
                world.flush_commands();
                let mut state = NodeState::from_world_mut(entity, world).unwrap();
                state.insert(0.0, 0.0, VarName::visible, false.into(), NodeKind::None);
                state.insert(*t, 0.0, VarName::visible, true.into(), NodeKind::None);
                state.insert(
                    *t + a.duration,
                    0.0,
                    VarName::visible,
                    false.into(),
                    NodeKind::None,
                );
                state.insert(*t, 0.0, VarName::t, 0.0.into(), NodeKind::None);
                state.insert(
                    *t + 0.0001,
                    a.duration,
                    VarName::t,
                    1.0.into(),
                    NodeKind::None,
                );
                for (var, value) in a.context.get_vars() {
                    state.insert(0.0, 0.0, var, value, NodeKind::None);
                }
                a.targets = vec![entity];
                end_t = *t + a.duration;
                *t += a.timeframe;
            }
            AnimAction::Wait(expression) => {
                *t += expression.get_f32(&a.context)?;
            }
        };
        Ok(end_t)
    }
}

impl<'w, 's> Animator<'w, 's> {
    pub fn new(context: Context<'w, 's>) -> Self {
        Self {
            targets: Vec::new(),
            context,
            duration: 1.0,
            timeframe: 0.0,
        }
    }
}

impl Default for AnimAction {
    fn default() -> Self {
        Self::Translate(Box::new(Expression::V2(0.0, 0.0)))
    }
}
impl Inject for AnimAction {
    fn move_inner(&mut self, source: &mut Self) {
        <Self as Injector<Self>>::inject_inner(self, source);
        <Self as Injector<Expression>>::inject_inner(self, source);
    }
    fn wrapper() -> Self {
        Self::List(vec![default()])
    }
}
impl Injector<Self> for AnimAction {
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        match self {
            AnimAction::Translate(..)
            | AnimAction::SetTarget(..)
            | AnimAction::AddTarget(..)
            | AnimAction::Duration(..)
            | AnimAction::Timeframe(..)
            | AnimAction::Wait(..)
            | AnimAction::Spawn(..) => default(),
            AnimAction::List(vec) => vec.into_iter().collect_vec(),
        }
    }
    fn get_inner(&self) -> Vec<&Box<Self>> {
        match self {
            AnimAction::Translate(..)
            | AnimAction::SetTarget(..)
            | AnimAction::AddTarget(..)
            | AnimAction::Duration(..)
            | AnimAction::Timeframe(..)
            | AnimAction::Wait(..)
            | AnimAction::Spawn(..) => default(),
            AnimAction::List(vec) => vec.into_iter().collect_vec(),
        }
    }
}
impl Injector<Expression> for AnimAction {
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Expression>> {
        match self {
            AnimAction::Translate(x)
            | AnimAction::SetTarget(x)
            | AnimAction::AddTarget(x)
            | AnimAction::Duration(x)
            | AnimAction::Wait(x)
            | AnimAction::Timeframe(x) => [x].into(),
            AnimAction::List(..) | AnimAction::Spawn(..) => default(),
        }
    }
    fn get_inner(&self) -> Vec<&Box<Expression>> {
        match self {
            AnimAction::Translate(x)
            | AnimAction::SetTarget(x)
            | AnimAction::AddTarget(x)
            | AnimAction::Duration(x)
            | AnimAction::Wait(x)
            | AnimAction::Timeframe(x) => [x].into(),
            AnimAction::List(..) | AnimAction::Spawn(..) => default(),
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
impl Show for Anim {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        prefix.show(ui);
        self.actions.show(None, context, ui)
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        DataFrameMut::new(self)
            .prefix(prefix)
            .body(|d, ui| d.actions.show_mut(None, ui))
            .ui(ui)
    }
}

impl DataFramed for AnimAction {
    fn show_name_mut(&mut self, ui: &mut Ui) -> bool {
        Selector::from_mut(self, ui)
    }
    fn has_header(&self) -> bool {
        match self {
            AnimAction::Translate(..)
            | AnimAction::SetTarget(..)
            | AnimAction::AddTarget(..)
            | AnimAction::Duration(..)
            | AnimAction::Timeframe(..)
            | AnimAction::Wait(..)
            | AnimAction::Spawn(..)
            | AnimAction::List(..) => false,
        }
    }
    fn has_body(&self) -> bool {
        match self {
            AnimAction::Translate(..)
            | AnimAction::SetTarget(..)
            | AnimAction::AddTarget(..)
            | AnimAction::Duration(..)
            | AnimAction::Timeframe(..)
            | AnimAction::Wait(..)
            | AnimAction::Spawn(..)
            | AnimAction::List(..) => true,
        }
    }
    fn show_header(&self, _: &Context, _: &mut Ui) {}
    fn show_header_mut(&mut self, _: &mut Ui) -> bool {
        false
    }
    fn show_body(&self, context: &Context, ui: &mut Ui) {
        match self {
            AnimAction::Translate(x)
            | AnimAction::SetTarget(x)
            | AnimAction::AddTarget(x)
            | AnimAction::Duration(x)
            | AnimAction::Wait(x)
            | AnimAction::Timeframe(x) => x.show(Some("x:"), context, ui),
            AnimAction::Spawn(m) => m.show(Some("material:"), context, ui),
            AnimAction::List(vec) => vec.show(Some("list:"), context, ui),
        }
    }
    fn show_body_mut(&mut self, ui: &mut Ui) -> bool {
        match self {
            AnimAction::Translate(x)
            | AnimAction::SetTarget(x)
            | AnimAction::AddTarget(x)
            | AnimAction::Duration(x)
            | AnimAction::Wait(x)
            | AnimAction::Timeframe(x) => x.show_mut(Some("x:"), ui),
            AnimAction::Spawn(m) => m.show_mut(Some("material:"), ui),
            AnimAction::List(vec) => vec.show_mut(Some("list:"), ui),
        }
    }
}
