use expression::Expression;

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumIter, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum PainterAction {
    Circle(Expression),
    Rectangle(Expression),
    Text(Expression),
    Hollow(Expression),
    Translate(Expression),
    Scale(Expression),
    Color(Expression),
    Repeat(Expression, Box<PainterAction>),
}

impl Default for PainterAction {
    fn default() -> Self {
        Self::Rectangle(Expression::V2(1.0, 1.0))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RMaterial {
    #[serde(default)]
    pub actions: Vec<PainterAction>,
}
