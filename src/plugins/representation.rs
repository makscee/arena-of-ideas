use super::*;

pub struct RepresentationPlugin;

impl Plugin for RepresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            Self::injector_system.run_if(
                in_state(GameState::Battle)
                    .or_else(in_state(GameState::Shop))
                    .or_else(in_state(GameState::HeroGallery))
                    .or_else(in_state(GameState::HeroEditor)),
            ),
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
        let t = get_play_head();
        let dragged = world
            .get_resource::<DraggedUnit>()
            .unwrap()
            .0
            .map(|(d, _)| d);
        for (entity, rep) in reps {
            let context = Context::from_owner(entity, world);
            let mapping: HashMap<VarName, VarValue> =
                HashMap::from_iter(rep.mapping.iter().filter_map(|(var, value)| {
                    match value.get_value(&context, world) {
                        Ok(value) => Some((*var, value)),
                        Err(_) => None,
                    }
                }));
            let mut state = VarState::get_mut(entity, world);
            for (var, value) in mapping {
                state.init(var, value);
            }

            let position = VarState::get_value(entity, VarName::Position, t, world)
                .and_then(|x| x.get_vec2())
                .unwrap_or_default();
            let rotation = VarState::get_value(entity, VarName::Rotation, t, world)
                .and_then(|x| x.get_float())
                .unwrap_or_default();
            let scale = VarState::get_value(entity, VarName::Size, t, world)
                .and_then(|x| x.get_vec2())
                .unwrap_or(Vec2::ONE);

            let mut transform = world.get_mut::<Transform>(entity).unwrap();
            if dragged != Some(entity) {
                transform.translation.x = position.x;
                transform.translation.y = position.y;
            }
            transform.rotation = Quat::from_rotation_z(rotation);
            transform.scale.x = scale.x;
            transform.scale.y = scale.y;
            rep.material.update(entity, world);
        }
    }
}
