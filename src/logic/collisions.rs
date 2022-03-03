use super::*;

impl Game {
    pub fn process_collisions(&mut self, unit: &mut Unit) {
        for other in &mut self.units {
            let delta_pos = other.position - unit.position;
            let penetration = unit.radius() + other.radius() - delta_pos.len();
            if penetration > Coord::ZERO {
                let dir = delta_pos.normalize_or_zero();
                unit.position -= dir * penetration / Coord::new(2.0);
                other.position += dir * penetration / Coord::new(2.0);
            }
        }
    }
}
