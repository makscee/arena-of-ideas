use expression::Expression;
use glam::vec2;

use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RMaterial {
    pub t: MaterialType,
    #[serde(default)]
    pub count: u32,
    #[serde(default)]
    pub modifiers: Vec<RModifier>,
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumIter, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum RModifier {
    Color(Expression),
    Offset(Expression),
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumIter, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum MaterialType {
    Shape {
        shape: Shape,
        #[serde(default)]
        modifiers: Vec<ShapeModifier>,
    },
    Text {
        text: Expression,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumIter, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum Shape {
    Rectangle { size: Expression },
    Circle { radius: Expression },
}

#[derive(Debug, Clone, Serialize, Deserialize, AsRefStr, EnumIter, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum ShapeModifier {
    Scale(Expression),
    Rotation(Expression),
    Color(Expression),
    Hollow(Expression),
    Thickness(Expression),
    Roundness(Expression),
    Alpha(Expression),
}

impl Default for RModifier {
    fn default() -> Self {
        RModifier::Offset(Expression::V2(0.0, 0.0))
    }
}
impl Default for Shape {
    fn default() -> Self {
        Shape::Rectangle {
            size: Expression::V(vec2(1.0, 1.0).into()),
        }
    }
}
impl Default for MaterialType {
    fn default() -> Self {
        Self::Shape {
            shape: default(),
            modifiers: default(),
        }
    }
}
impl Default for ShapeModifier {
    fn default() -> Self {
        Self::Rotation(Expression::Zero)
    }
}
