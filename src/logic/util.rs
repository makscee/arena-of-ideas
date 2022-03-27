use super::*;

pub fn distance_between_units(a: &Unit, b: &Unit) -> Coord {
    (a.position - b.position).len() - a.radius - b.radius
}
