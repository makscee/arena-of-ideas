use super::*;

pub fn distance_between_units(a: &Unit, b: &Unit) -> Coord {
    (a.position - b.position).abs()
}

pub fn pos_to_world(position: Position) -> Vec2<R32> {
    vec2(r32(position as f32), R32::ZERO)
}
