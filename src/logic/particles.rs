use super::*;

impl Logic {
    pub fn process_particles(&mut self) {
        for particle in &mut self.model.particles {
            particle.delay -= self.delta_time;
            if particle.delay > Time::new(0.0) {
                continue;
            }
            particle.time_left -= self.delta_time;
            let parent = particle.parent.and_then(|parent| {
                self.model
                    .units
                    .get(&parent)
                    .or(self.model.dead_units.get(&parent))
            });
            let partner = particle.partner.and_then(|partner| {
                self.model
                    .units
                    .get(&partner)
                    .or(self.model.dead_units.get(&partner))
            });
            let mut parameters = &mut particle.render_config.parameters;

            if let Some(color) = particle.color {
                parameters.0.extend(HashMap::from([(
                    "u_color".to_string(),
                    ShaderParameter::Color(color),
                )]));
            }
            if let Some(parent) = parent {
                if particle.follow {
                    particle.position = parent.position.to_world();
                }

                parameters.0.extend(HashMap::from([(
                    "u_parent_position".to_string(),
                    ShaderParameter::Vec2(parent.position.to_world().map(|x| x.as_f32())),
                )]));
                parameters.0.extend(HashMap::from([(
                    "u_parent_radius".to_string(),
                    ShaderParameter::Float(parent.stats.radius.as_f32()),
                )]));
                parameters.0.extend(HashMap::from([(
                    "u_parent_random".to_string(),
                    ShaderParameter::Float(parent.random_number.as_f32()),
                )]));
                parameters.0.extend(HashMap::from([(
                    "u_parent_faction".to_string(),
                    ShaderParameter::Float(match parent.faction {
                        Faction::Player => 1.0,
                        Faction::Enemy => -1.0,
                    }),
                )]));
            }
            if let Some(partner) = partner {
                parameters.0.extend(HashMap::from([(
                    "u_partner_position".to_string(),
                    ShaderParameter::Vec2(partner.position.to_world().map(|x| x.as_f32())),
                )]));
            }
        }
        self.model
            .particles
            .retain(|particle| particle.time_left > Time::new(0.0))
    }
}
