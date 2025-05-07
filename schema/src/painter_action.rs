use expression::Expression;

use super::*;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumIter, PartialEq, Hash)]
#[serde(deny_unknown_fields)]
pub enum PainterAction {
    paint,
    circle(Box<Expression>),
    rectangle(Box<Expression>),
    curve {
        thickness: Box<Expression>,
        curvature: Box<Expression>,
    },
    text(Box<Expression>),
    hollow(Box<Expression>),
    translate(Box<Expression>),
    rotate(Box<Expression>),
    scale_mesh(Box<Expression>),
    scale_rect(Box<Expression>),
    color(Box<Expression>),
    alpha(Box<Expression>),
    feathering(Box<Expression>),
    repeat(Box<Expression>, Box<PainterAction>),
    list(Vec<Box<PainterAction>>),
}

impl Default for PainterAction {
    fn default() -> Self {
        Self::circle(Box::new(Expression::f32(1.0)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Hash)]
#[serde(deny_unknown_fields)]
pub struct Material(pub Vec<PainterAction>);
