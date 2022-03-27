use super::*;

impl Logic<'_> {
    pub fn process_collisions(&mut self) {
        self.process_units(Self::process_unit_collisions);
    }
    pub fn process_unit_collisions(&mut self, unit: &mut Unit) {
        for other in &mut self.model.units {
            let delta_pos = other.position - unit.position;
            let penetration = unit.radius + other.radius - delta_pos.len();
            if penetration > Coord::ZERO {
                let mut dir = delta_pos.normalize_or_zero();
                if dir == Vec2::ZERO {
                    dir = vec2(Coord::ONE, Coord::ZERO)
                        .rotate(Coord::new(global_rng().gen_range(0.0..2.0 * f32::PI)));
                }
                unit.position -= dir * penetration / Coord::new(2.0);
                other.position += dir * penetration / Coord::new(2.0);
            }
        }
    }
}
