use super::*;

impl Logic<'_> {
    pub fn process_particles(&mut self) {
        for particle in &mut self.model.particles {
            if let Some(time) = &mut particle.time_left {
                *time -= self.delta_time;
            }
            let parent = particle
                .parent
                .and_then(|parent| self.model.units.get(&parent))
                .map(|unit| unit.position);
            if let Some(parent) = parent {
                particle.position = parent;
            }
        }
        self.model.particles.retain(|particle| {
            particle
                .time_left
                .map(|time| time > Time::new(0.0))
                .unwrap_or(true)
        })
    }
}
