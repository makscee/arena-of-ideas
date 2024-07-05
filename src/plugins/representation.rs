use super::*;

pub struct RepresentationPlugin;

impl Plugin for RepresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::injector_system);
    }
}

impl RepresentationPlugin {
    pub fn get_by_id(id: u64) -> Representation {
        if id == 0 {
            return default();
        }
        ron::from_str(&TRepresentation::filter_by_id(id).unwrap().data).unwrap()
    }
    fn injector_system(world: &mut World) {
        let reps = world
            .query::<(Entity, &Representation)>()
            .iter(world)
            .map(|(e, r)| (e, r.clone()))
            .collect_vec();
        for (_, rep) in reps {
            rep.update(world);
        }
    }
}
