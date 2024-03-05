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
        let dragged = world
            .get_resource::<DraggedUnit>()
            .unwrap()
            .0
            .map(|(d, _)| d);
        for (_, rep) in reps {
            rep.update(dragged, world);
        }
    }
}
