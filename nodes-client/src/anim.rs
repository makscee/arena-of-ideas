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
    List(Vec<Box<Self>>),
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
        Self {
            actions: actions.into_iter().map(|a| Box::new(a)).collect(),
        }
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
    pub fn new_set(entity: Entity, var: VarName, value: VarValue) -> Self {
        Self {
            entity,
            duration: 0.0,
            timeframe: 0.0,
            vars: [(var, value)].into(),
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
            AnimAction::List(vec) => {
                for aa in vec {
                    aa.apply(a)?;
                }
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
            | AnimAction::Timeframe(..) => default(),
            AnimAction::List(vec) => vec.into_iter().collect_vec(),
        }
    }
    fn get_inner(&self) -> Vec<&Box<Self>> {
        match self {
            AnimAction::Translate(..)
            | AnimAction::SetTarget(..)
            | AnimAction::AddTarget(..)
            | AnimAction::Duration(..)
            | AnimAction::Timeframe(..) => default(),
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
            | AnimAction::Timeframe(x) => [x].into(),
            AnimAction::List(..) => default(),
        }
    }
    fn get_inner(&self) -> Vec<&Box<Expression>> {
        match self {
            AnimAction::Translate(x)
            | AnimAction::SetTarget(x)
            | AnimAction::AddTarget(x)
            | AnimAction::Duration(x)
            | AnimAction::Timeframe(x) => [x].into(),
            AnimAction::List(..) => default(),
        }
    }
}
impl ToCstr for AnimAction {
    fn cstr(&self) -> Cstr {
        self.as_ref().cstr_c(PURPLE)
    }
}
impl Show for AnimAction {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        format!("{}{}", prefix.unwrap_or_default(), self.cstr()).label(ui);
        let inner = <Self as Injector<Expression>>::get_inner(self);
        if !inner.is_empty() {
            for i in inner {
                i.show(None, context, ui);
            }
        };
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        CollapsingSelector::ui(self, prefix, ui, |v, ui| match v {
            AnimAction::Translate(x)
            | AnimAction::SetTarget(x)
            | AnimAction::AddTarget(x)
            | AnimAction::Duration(x)
            | AnimAction::Timeframe(x) => x.show_mut(Some("v:"), ui),
            AnimAction::List(vec) => vec.show_mut(prefix, ui),
        })
    }
}
impl Show for Anim {
    fn show(&self, prefix: Option<&str>, context: &Context, ui: &mut Ui) {
        prefix.show(None, context, ui);
        self.actions.show(None, context, ui);
    }
    fn show_mut(&mut self, prefix: Option<&str>, ui: &mut Ui) -> bool {
        self.actions.show_mut(prefix, ui)
    }
}
