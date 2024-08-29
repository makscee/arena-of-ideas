use super::*;

pub struct RepresentationPlugin;

impl Plugin for RepresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::injector_system);
    }
}

impl RepresentationPlugin {
    pub fn get_by_id(id: String) -> Option<Representation> {
        TRepresentation::find_by_id(id).and_then(|r| ron::from_str(&r.data).ok())
    }
    fn injector_system(world: &mut World) {
        let reps = world
            .query::<(Entity, &Representation)>()
            .iter(world)
            .map(|(e, r)| (e, r.clone()))
            .collect_vec();
        for (entity, rep) in reps {
            rep.update(entity, Context::new_play(entity), world);
        }
    }
}
