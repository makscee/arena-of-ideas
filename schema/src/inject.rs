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
    fn inject_inner(&mut self, source: &mut Self) {
        let mut source_inner = source.get_inner_mut();
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

impl Injector<Self> for Expression {
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        match self {
            Expression::One
            | Expression::Zero
            | Expression::GT
            | Expression::Var(..)
            | Expression::V(..)
            | Expression::S(..)
            | Expression::F(..)
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
            | Expression::Sqr(x) => [x].into(),
            Expression::Macro(a, b)
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
            | Expression::LessThen(a, b) => [a, b].into(),
            Expression::If(a, b, c) => [a, b, c].into(),
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
            | PainterAction::Scale(x)
            | PainterAction::Color(x)
            | PainterAction::Alpha(x)
            | PainterAction::Repeat(x, ..) => [x].into(),
        }
    }
}
impl Injector<Self> for PainterAction {
    fn get_inner_mut(&mut self) -> Vec<&mut Box<Self>> {
        match self {
            PainterAction::Paint
            | PainterAction::Circle(..)
            | PainterAction::Rectangle(..)
            | PainterAction::Text(..)
            | PainterAction::Hollow(..)
            | PainterAction::Translate(..)
            | PainterAction::Rotate(..)
            | PainterAction::Scale(..)
            | PainterAction::Color(..)
            | PainterAction::Alpha(..) => default(),
            PainterAction::Repeat(_x, p) => [p].into(),
            PainterAction::List(vec) => vec.into_iter().collect_vec(),
        }
    }
}