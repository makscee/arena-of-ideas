use super::*;

pub fn distance_between_units(a: &Unit, b: &Unit) -> Coord {
    (a.position.x - b.position.x).abs()
}

pub fn pos_to_world(position: Position) -> Vec2<R32> {
    position.map(|x| r32(x as f32))
}
