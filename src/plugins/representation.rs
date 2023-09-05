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
            let mut position = world
                .get::<VarState>(entity)
                .and_then(|x| {
                    x.get_value_at(VarName::Position, t)
                        .map(|x| x.get_vec2().unwrap())
                        .ok()
                })
                .unwrap_or_default();
            let context = Context::from_owner(entity, world);
            for (key, value) in rep.mapping.iter() {
                match key {
                    VarName::Position => {
                        position = value.get_vec2(&context, world).unwrap();
                    }
                    _ => continue,
                };
            }
            let mut transform = world.get_mut::<Transform>(entity).unwrap();
            transform.translation.x = position.x;
            transform.translation.y = position.y;
            rep.material.update(entity, world);
        }
    }
}
