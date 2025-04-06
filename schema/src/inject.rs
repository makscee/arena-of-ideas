use std::mem;

use itertools::Itertools;

use super::*;

pub trait Inject: Injector<Self> {
    fn move_inner(&mut self, source: &mut Self) {}
    fn wrapper() -> Option<Self> {
        None
    }
    fn wrap(&mut self) {
        let Some(mut wrapper) = Self::wrapper() else {
            return;
        };
        mem::swap(wrapper.get_inner_mut()[0], self);
        *self = wrapper;
    }
}

pub trait Injector<T>: Sized {
    fn get_inner_mut<'a>(&'a mut self) -> Vec<&'a mut T>;
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

impl<T> Injector<Self> for Vec<T>
where
    T: Default + Serialize + DeserializeOwned,
{
    fn get_inner_mut(&mut self) -> Vec<&mut Self> {
        default()
    }

    fn get_inner(&self) -> Vec<&Box<Self>> {
        default()
    }
}
impl<T> Inject for Vec<T>
where
    T: Default + Serialize + DeserializeOwned,
{
    fn move_inner(&mut self, _: &mut Self) {}

    fn wrapper() -> Option<Self> {
        Some([default()].into())
    }
}

impl Inject for Expression {
    fn move_inner(&mut self, source: &mut Self) {
        <Expression as Injector<Expression>>::inject_inner(self, source);
        <Expression as Injector<f32>>::inject_inner(self, source);
    }
    fn wrapper() -> Option<Self> {
        Some(Self::abs(default()))
    }
}

impl Inject for PainterAction {
    fn move_inner(&mut self, source: &mut Self) {
        <Self as Injector<Self>>::inject_inner(self, source);
        <Self as Injector<Expression>>::inject_inner(self, source);
    }
    fn wrapper() -> Option<Self> {
        Some(Self::repeat(Box::new(Expression::i32(1)), default()))
    }
}

impl Inject for Action {
    fn move_inner(&mut self, source: &mut Self) {
        <Self as Injector<Self>>::inject_inner(self, source);
        <Self as Injector<Expression>>::inject_inner(self, source);
    }
    fn wrapper() -> Option<Self> {
        Some(Self::repeat(Box::new(Expression::i32(1)), default()))
    }
}

impl Injector<Self> for Expression {
    fn get_inner_mut(&mut self) -> Vec<&mut Self> {
        match self {
            Expression::one
            | Expression::zero
            | Expression::pi
            | Expression::pi2
            | Expression::gt
            | Expression::unit_size
            | Expression::all_units
            | Expression::all_ally_units
            | Expression::all_other_ally_units
            | Expression::adjacent_ally_units
            | Expression::adjacent_back
            | Expression::adjacent_front
            | Expression::all_enemy_units
            | Expression::owner
            | Expression::target
            | Expression::var(..)
            | Expression::value(..)
            | Expression::string(..)
            | Expression::f32(..)
            | Expression::f32_slider(..)
            | Expression::i32(..)
            | Expression::bool(..)
            | Expression::vec2(..)
            | Expression::color(..) => default(),
            Expression::sin(x)
            | Expression::cos(x)
            | Expression::even(x)
            | Expression::abs(x)
            | Expression::floor(x)
            | Expression::ceil(x)
            | Expression::fract(x)
            | Expression::unit_vec(x)
            | Expression::rand(x)
            | Expression::random_unit(x)
            | Expression::to_f32(x)
            | Expression::state_var(x, _)
            | Expression::sqr(x) => [x.as_mut()].into(),
            Expression::str_macro(a, b)
            | Expression::vec2_ee(a, b)
            | Expression::sum(a, b)
            | Expression::sub(a, b)
            | Expression::mul(a, b)
            | Expression::div(a, b)
            | Expression::max(a, b)
            | Expression::min(a, b)
            | Expression::r#mod(a, b)
            | Expression::and(a, b)
            | Expression::or(a, b)
            | Expression::equals(a, b)
            | Expression::greater_then(a, b)
            | Expression::less_then(a, b)
            | Expression::fallback(a, b) => [a.as_mut(), b.as_mut()].into(),
            Expression::oklch(a, b, c) | Expression::r#if(a, b, c) => {
                [a.as_mut(), b.as_mut(), c.as_mut()].into()
            }
        }
    }
    fn get_inner(&self) -> Vec<&Box<Self>> {
        match self {
            Expression::one
            | Expression::zero
            | Expression::pi
            | Expression::pi2
            | Expression::gt
            | Expression::unit_size
            | Expression::all_units
            | Expression::all_ally_units
            | Expression::all_other_ally_units
            | Expression::adjacent_ally_units
            | Expression::adjacent_back
            | Expression::adjacent_front
            | Expression::all_enemy_units
            | Expression::owner
            | Expression::target
            | Expression::var(..)
            | Expression::value(..)
            | Expression::string(..)
            | Expression::f32(..)
            | Expression::f32_slider(..)
            | Expression::i32(..)
            | Expression::bool(..)
            | Expression::vec2(..)
            | Expression::color(..) => default(),
            Expression::sin(x)
            | Expression::cos(x)
            | Expression::even(x)
            | Expression::abs(x)
            | Expression::floor(x)
            | Expression::ceil(x)
            | Expression::fract(x)
            | Expression::unit_vec(x)
            | Expression::rand(x)
            | Expression::random_unit(x)
            | Expression::to_f32(x)
            | Expression::state_var(x, _)
            | Expression::sqr(x) => [x].into(),
            Expression::str_macro(a, b)
            | Expression::vec2_ee(a, b)
            | Expression::sum(a, b)
            | Expression::sub(a, b)
            | Expression::mul(a, b)
            | Expression::div(a, b)
            | Expression::max(a, b)
            | Expression::min(a, b)
            | Expression::r#mod(a, b)
            | Expression::and(a, b)
            | Expression::or(a, b)
            | Expression::equals(a, b)
            | Expression::greater_then(a, b)
            | Expression::less_then(a, b)
            | Expression::fallback(a, b) => [a, b].into(),
            Expression::oklch(a, b, c) | Expression::r#if(a, b, c) => [a, b, c].into(),
        }
    }
}

impl Injector<f32> for Expression {
    fn get_inner_mut(&mut self) -> Vec<&mut f32> {
        match self {
            Expression::f32_slider(v) | Expression::f32(v) => [v].into(),
            _ => default(),
        }
    }

    fn get_inner(&self) -> Vec<&Box<f32>> {
        todo!()
    }
}

impl Injector<Expression> for PainterAction {
    fn get_inner_mut(&mut self) -> Vec<&mut Expression> {
        match self {
            PainterAction::list(..) | PainterAction::paint => default(),
            PainterAction::circle(x)
            | PainterAction::rectangle(x)
            | PainterAction::text(x)
            | PainterAction::hollow(x)
            | PainterAction::translate(x)
            | PainterAction::rotate(x)
            | PainterAction::scale_mesh(x)
            | PainterAction::scale_rect(x)
            | PainterAction::color(x)
            | PainterAction::alpha(x)
            | PainterAction::feathering(x)
            | PainterAction::repeat(x, ..) => [x.as_mut()].into(),
            PainterAction::curve {
                thickness,
                curvature,
            } => [thickness.as_mut(), curvature.as_mut()].into(),
        }
    }
    fn get_inner(&self) -> Vec<&Box<Expression>> {
        match self {
            PainterAction::list(..) | PainterAction::paint => default(),
            PainterAction::circle(x)
            | PainterAction::rectangle(x)
            | PainterAction::text(x)
            | PainterAction::hollow(x)
            | PainterAction::translate(x)
            | PainterAction::rotate(x)
            | PainterAction::scale_mesh(x)
            | PainterAction::scale_rect(x)
            | PainterAction::color(x)
            | PainterAction::alpha(x)
            | PainterAction::feathering(x)
            | PainterAction::repeat(x, ..) => [x].into(),
            PainterAction::curve {
                thickness,
                curvature,
            } => [thickness, curvature].into(),
        }
    }
}
impl Injector<Self> for PainterAction {
    fn resize_inner(&mut self, size: usize) {
        match self {
            PainterAction::list(vec) => vec.resize(size, default()),
            _ => {}
        }
    }
    fn get_inner_mut(&mut self) -> Vec<&mut Self> {
        match self {
            PainterAction::paint
            | PainterAction::circle(..)
            | PainterAction::rectangle(..)
            | PainterAction::curve { .. }
            | PainterAction::text(..)
            | PainterAction::hollow(..)
            | PainterAction::translate(..)
            | PainterAction::rotate(..)
            | PainterAction::scale_mesh(..)
            | PainterAction::scale_rect(..)
            | PainterAction::color(..)
            | PainterAction::feathering(..)
            | PainterAction::alpha(..) => default(),
            PainterAction::repeat(_x, p) => [p.as_mut()].into(),
            PainterAction::list(vec) => vec.into_iter().map(|v| v.as_mut()).collect_vec(),
        }
    }
    fn get_inner(&self) -> Vec<&Box<Self>> {
        match self {
            PainterAction::paint
            | PainterAction::circle(..)
            | PainterAction::rectangle(..)
            | PainterAction::curve { .. }
            | PainterAction::text(..)
            | PainterAction::hollow(..)
            | PainterAction::translate(..)
            | PainterAction::rotate(..)
            | PainterAction::scale_mesh(..)
            | PainterAction::scale_rect(..)
            | PainterAction::color(..)
            | PainterAction::feathering(..)
            | PainterAction::alpha(..) => default(),
            PainterAction::repeat(_x, p) => [p].into(),
            PainterAction::list(vec) => vec.into_iter().collect_vec(),
        }
    }
}

impl Injector<Self> for Action {
    fn get_inner_mut(&mut self) -> Vec<&mut Self> {
        match self {
            Action::noop
            | Action::debug(..)
            | Action::set_value(..)
            | Action::add_value(..)
            | Action::subtract_value(..)
            | Action::add_target(..)
            | Action::deal_damage
            | Action::heal_damage
            | Action::apply_status
            | Action::use_ability => default(),
            Action::repeat(_, vec) => vec.into_iter().map(|v| v.as_mut()).collect_vec(),
        }
    }
    fn get_inner(&self) -> Vec<&Box<Self>> {
        match self {
            Action::noop
            | Action::debug(..)
            | Action::set_value(..)
            | Action::add_value(..)
            | Action::subtract_value(..)
            | Action::add_target(..)
            | Action::deal_damage
            | Action::heal_damage
            | Action::apply_status
            | Action::use_ability => default(),
            Action::repeat(_, vec) => vec.into_iter().collect_vec(),
        }
    }
}
impl Injector<Expression> for Action {
    fn get_inner_mut(&mut self) -> Vec<&mut Expression> {
        match self {
            Action::noop
            | Action::deal_damage
            | Action::heal_damage
            | Action::apply_status
            | Action::use_ability => default(),
            Action::debug(x)
            | Action::set_value(x)
            | Action::add_value(x)
            | Action::subtract_value(x)
            | Action::add_target(x)
            | Action::repeat(x, _) => [x.as_mut()].into(),
        }
    }
    fn get_inner(&self) -> Vec<&Box<Expression>> {
        match self {
            Action::noop
            | Action::deal_damage
            | Action::heal_damage
            | Action::apply_status
            | Action::use_ability => default(),
            Action::debug(x)
            | Action::set_value(x)
            | Action::add_value(x)
            | Action::subtract_value(x)
            | Action::add_target(x)
            | Action::repeat(x, _) => [x].into(),
        }
    }
}
impl Inject for Trigger {}
impl Injector<Self> for Trigger {
    fn get_inner_mut(&mut self) -> Vec<&mut Self> {
        default()
    }
    fn get_inner(&self) -> Vec<&Box<Self>> {
        default()
    }
}
