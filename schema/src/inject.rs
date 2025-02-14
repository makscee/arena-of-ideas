use std::mem;

use itertools::Itertools;

use super::*;

pub trait Inject: Injector<Self> {
    fn move_inner(&mut self, source: &mut Self);
    fn wrapper() -> Self;
    fn wrap(&mut self) {
        let mut wrapper = Self::wrapper();
        mem::swap(wrapper.get_inner_mut()[0].as_mut(), self);
        *self = wrapper;
    }
}

pub trait Injector<T>: Sized {
    fn get_inner_mut(&mut self) -> Vec<&mut Box<T>>;
    fn get_inner(&self) -> Vec<&Box<T>>;
    fn resize_inner(&mut self, _size: usize) {}
    fn inject_inner(&mut self, source: &mut Self) {
        let mut source_inner = source.get_inner_mut();
        self.resize_inner(source_inner.len());
        for (ind, i) in self.get_inner_mut().iter_mut().enumerate() {
            if let Some(d) = source_inner.get_mut(ind) {
                mem::swap(*i, *d);
            }
        }
    }
}

impl Inject for Expression {
    fn move_inner(&mut self, source: &mut Self) {
        <Expression as Injector<Expression>>::inject_inner(self, source);
    }
    fn wrapper() -> Self {
        Self::Abs(default())
    }
}

impl Inject for PainterAction {
    fn move_inner(&mut self, source: &mut Self) {
        <Self as Injector<Self>>::inject_inner(self, source);
        <Self as Injector<Expression>>::inject_inner(self, source);
    }
    fn wrapper() -> Self {
        Self::Repeat(Box::new(Expression::I(1)), default())
    }
}

impl Inject for Action {
    fn move_inner(&mut self, source: &mut Self) {
        <Self as Injector<Self>>::inject_inner(self, source);
        <Self as Injector<Expression>>::inject_inner(self, source);
    }
    fn wrapper() -> Self {
        Self::Repeat(Box::new(Expression::I(1)), default())
    }
}

impl Injector<Self> for Expression {
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        match self {
            Expression::One
            | Expression::Zero
            | Expression::PI
            | Expression::PI2
            | Expression::GT
            | Expression::UnitSize
            | Expression::AllUnits
            | Expression::AllAllyUnits
            | Expression::AllOtherAllyUnits
            | Expression::AdjacentAllyUnits
            | Expression::AdjacentBack
            | Expression::AdjacentFront
            | Expression::AllEnemyUnits
            | Expression::Owner
            | Expression::Target
            | Expression::Var(..)
            | Expression::V(..)
            | Expression::S(..)
            | Expression::F(..)
            | Expression::FSlider(..)
            | Expression::I(..)
            | Expression::B(..)
            | Expression::V2(..)
            | Expression::C(..) => default(),
            Expression::Sin(x)
            | Expression::Cos(x)
            | Expression::Even(x)
            | Expression::Abs(x)
            | Expression::Floor(x)
            | Expression::Ceil(x)
            | Expression::Fract(x)
            | Expression::UnitVec(x)
            | Expression::Rand(x)
            | Expression::RandomUnit(x)
            | Expression::ToF(x)
            | Expression::StateVar(x, _)
            | Expression::Sqr(x) => [x].into(),
            Expression::Macro(a, b)
            | Expression::V2EE(a, b)
            | Expression::Sum(a, b)
            | Expression::Sub(a, b)
            | Expression::Mul(a, b)
            | Expression::Div(a, b)
            | Expression::Max(a, b)
            | Expression::Min(a, b)
            | Expression::Mod(a, b)
            | Expression::And(a, b)
            | Expression::Or(a, b)
            | Expression::Equals(a, b)
            | Expression::GreaterThen(a, b)
            | Expression::LessThen(a, b)
            | Expression::Fallback(a, b) => [a, b].into(),
            Expression::Oklch(a, b, c) | Expression::If(a, b, c) => [a, b, c].into(),
        }
    }
    fn get_inner(&self) -> Vec<&Box<Self>> {
        match self {
            Expression::One
            | Expression::Zero
            | Expression::PI
            | Expression::PI2
            | Expression::GT
            | Expression::UnitSize
            | Expression::AllUnits
            | Expression::AllAllyUnits
            | Expression::AllOtherAllyUnits
            | Expression::AdjacentAllyUnits
            | Expression::AdjacentBack
            | Expression::AdjacentFront
            | Expression::AllEnemyUnits
            | Expression::Owner
            | Expression::Target
            | Expression::Var(..)
            | Expression::V(..)
            | Expression::S(..)
            | Expression::F(..)
            | Expression::FSlider(..)
            | Expression::I(..)
            | Expression::B(..)
            | Expression::V2(..)
            | Expression::C(..) => default(),
            Expression::Sin(x)
            | Expression::Cos(x)
            | Expression::Even(x)
            | Expression::Abs(x)
            | Expression::Floor(x)
            | Expression::Ceil(x)
            | Expression::Fract(x)
            | Expression::UnitVec(x)
            | Expression::Rand(x)
            | Expression::RandomUnit(x)
            | Expression::ToF(x)
            | Expression::StateVar(x, _)
            | Expression::Sqr(x) => [x].into(),
            Expression::Macro(a, b)
            | Expression::V2EE(a, b)
            | Expression::Sum(a, b)
            | Expression::Sub(a, b)
            | Expression::Mul(a, b)
            | Expression::Div(a, b)
            | Expression::Max(a, b)
            | Expression::Min(a, b)
            | Expression::Mod(a, b)
            | Expression::And(a, b)
            | Expression::Or(a, b)
            | Expression::Equals(a, b)
            | Expression::GreaterThen(a, b)
            | Expression::LessThen(a, b)
            | Expression::Fallback(a, b) => [a, b].into(),
            Expression::Oklch(a, b, c) | Expression::If(a, b, c) => [a, b, c].into(),
        }
    }
}

impl Injector<Expression> for PainterAction {
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Expression>> {
        match self {
            PainterAction::List(..) | PainterAction::Paint => default(),
            PainterAction::Circle(x)
            | PainterAction::Rectangle(x)
            | PainterAction::Text(x)
            | PainterAction::Hollow(x)
            | PainterAction::Translate(x)
            | PainterAction::Rotate(x)
            | PainterAction::ScaleMesh(x)
            | PainterAction::ScaleRect(x)
            | PainterAction::Color(x)
            | PainterAction::Alpha(x)
            | PainterAction::Feathering(x)
            | PainterAction::Repeat(x, ..) => [x].into(),
            PainterAction::Curve {
                thickness,
                curvature,
            } => [thickness, curvature].into(),
        }
    }
    fn get_inner(&self) -> Vec<&Box<Expression>> {
        match self {
            PainterAction::List(..) | PainterAction::Paint => default(),
            PainterAction::Circle(x)
            | PainterAction::Rectangle(x)
            | PainterAction::Text(x)
            | PainterAction::Hollow(x)
            | PainterAction::Translate(x)
            | PainterAction::Rotate(x)
            | PainterAction::ScaleMesh(x)
            | PainterAction::ScaleRect(x)
            | PainterAction::Color(x)
            | PainterAction::Alpha(x)
            | PainterAction::Feathering(x)
            | PainterAction::Repeat(x, ..) => [x].into(),
            PainterAction::Curve {
                thickness,
                curvature,
            } => [thickness, curvature].into(),
        }
    }
}
impl Injector<Self> for PainterAction {
    fn resize_inner(&mut self, size: usize) {
        match self {
            PainterAction::List(vec) => vec.resize(size, default()),
            _ => {}
        }
    }
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        match self {
            PainterAction::Paint
            | PainterAction::Circle(..)
            | PainterAction::Rectangle(..)
            | PainterAction::Curve { .. }
            | PainterAction::Text(..)
            | PainterAction::Hollow(..)
            | PainterAction::Translate(..)
            | PainterAction::Rotate(..)
            | PainterAction::ScaleMesh(..)
            | PainterAction::ScaleRect(..)
            | PainterAction::Color(..)
            | PainterAction::Feathering(..)
            | PainterAction::Alpha(..) => default(),
            PainterAction::Repeat(_x, p) => [p].into(),
            PainterAction::List(vec) => vec.into_iter().collect_vec(),
        }
    }
    fn get_inner(&self) -> Vec<&Box<Self>> {
        match self {
            PainterAction::Paint
            | PainterAction::Circle(..)
            | PainterAction::Rectangle(..)
            | PainterAction::Curve { .. }
            | PainterAction::Text(..)
            | PainterAction::Hollow(..)
            | PainterAction::Translate(..)
            | PainterAction::Rotate(..)
            | PainterAction::ScaleMesh(..)
            | PainterAction::ScaleRect(..)
            | PainterAction::Color(..)
            | PainterAction::Feathering(..)
            | PainterAction::Alpha(..) => default(),
            PainterAction::Repeat(_x, p) => [p].into(),
            PainterAction::List(vec) => vec.into_iter().collect_vec(),
        }
    }
}

impl Injector<Self> for Action {
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        match self {
            Action::Noop
            | Action::Debug(..)
            | Action::SetValue(..)
            | Action::AddValue(..)
            | Action::SubtractValue(..)
            | Action::AddTarget(..)
            | Action::DealDamage
            | Action::HealDamage
            | Action::UseAbility => default(),
            Action::Repeat(_, vec) => vec.into_iter().collect_vec(),
        }
    }
    fn get_inner(&self) -> Vec<&Box<Self>> {
        match self {
            Action::Noop
            | Action::Debug(..)
            | Action::SetValue(..)
            | Action::AddValue(..)
            | Action::SubtractValue(..)
            | Action::AddTarget(..)
            | Action::DealDamage
            | Action::HealDamage
            | Action::UseAbility => default(),
            Action::Repeat(_, vec) => vec.into_iter().collect_vec(),
        }
    }
}
impl Injector<Expression> for Action {
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Expression>> {
        match self {
            Action::Noop | Action::DealDamage | Action::HealDamage | Action::UseAbility => {
                default()
            }
            Action::Debug(x)
            | Action::SetValue(x)
            | Action::AddValue(x)
            | Action::SubtractValue(x)
            | Action::AddTarget(x)
            | Action::Repeat(x, _) => [x].into(),
        }
    }
    fn get_inner(&self) -> Vec<&Box<Expression>> {
        match self {
            Action::Noop | Action::DealDamage | Action::HealDamage | Action::UseAbility => {
                default()
            }
            Action::Debug(x)
            | Action::SetValue(x)
            | Action::AddValue(x)
            | Action::SubtractValue(x)
            | Action::AddTarget(x)
            | Action::Repeat(x, _) => [x].into(),
        }
    }
}
impl Inject for Trigger {
    fn move_inner(&mut self, _source: &mut Self) {}
    fn wrapper() -> Self {
        Self::default()
    }
}
impl Injector<Self> for Trigger {
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        default()
    }
    fn get_inner(&self) -> Vec<&Box<Self>> {
        default()
    }
}
