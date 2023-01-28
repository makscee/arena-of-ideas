use super::*;

pub struct Position(pub Vec2<f32>);

impl Default for Position {
    fn default() -> Self {
        Self(Vec2::ZERO)
    }
}
