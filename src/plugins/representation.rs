use super::*;

pub struct RepresentationPlugin;

impl Plugin for RepresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::injector_system
                .run_if(in_state(GameState::Battle).or_else(in_state(GameState::Shop))),
        );
    }
}

impl RepresentationPlugin {
    fn injector_system(world: &mut World) {
        let reps = world
            .query::<(Entity, &Representation)>()
            .iter(world)
            .map(|(e, r)| (e, r.clone()))
            .collect_vec();
        let t = GameTimer::get_mut(world).get_t();
        for (entity, rep) in reps {
            let mut position = VarState::get_value(entity, VarName::Position, t, world)
                .map(|x| x.get_vec2().unwrap())
                .unwrap_or_default();
            let mut rotation = VarState::get_value(entity, VarName::Rotation, t, world)
                .map(|x| x.get_float().unwrap())
                .unwrap_or_default();
            let mut scale = VarState::get_value(entity, VarName::Size, t, world)
                .map(|x| x.get_vec2().unwrap())
                .unwrap_or(Vec2::ONE);
            let context = Context::from_owner(entity, world);
            for (key, value) in rep.mapping.iter() {
                match key {
                    VarName::Position => {
                        position = value.get_vec2(&context, world).unwrap();
                    }
                    VarName::Rotation => {
                        rotation = value.get_float(&context, world).unwrap();
                    }
                    VarName::Scale => {
                        scale = value.get_vec2(&context, world).unwrap();
                    }
                    _ => continue,
                };
            }
            let mut transform = world.get_mut::<Transform>(entity).unwrap();
            transform.translation.x = position.x;
            transform.translation.y = position.y;
            transform.rotation = Quat::from_rotation_z(rotation);
            transform.scale.x = scale.x;
            transform.scale.y = scale.y;
            rep.material.update(entity, world);
        }
    }
}
