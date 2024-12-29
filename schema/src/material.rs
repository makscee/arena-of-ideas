use expression::Expression;

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumIter, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum PainterAction {
    Paint,
    Circle(Box<Expression>),
    Rectangle(Box<Expression>),
    Text(Box<Expression>),
    Hollow(Box<Expression>),
    Translate(Box<Expression>),
    Rotate(Box<Expression>),
    Scale(Box<Expression>),
    Color(Box<Expression>),
    Alpha(Box<Expression>),
    Repeat(Box<Expression>, Box<PainterAction>),
    List(Vec<Box<PainterAction>>),
}

impl Default for PainterAction {
    fn default() -> Self {
        Self::Rectangle(Box::new(Expression::V2(1.0, 1.0)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Material(pub Vec<Box<PainterAction>>);
