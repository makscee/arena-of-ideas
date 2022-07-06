use super::*;

pub fn distance_between_units(a: &Unit, b: &Unit) -> Coord {
    (a.position.x - b.position.x).abs()
}
