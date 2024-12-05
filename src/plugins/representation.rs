use super::*;

pub struct RepresentationPlugin;

impl Plugin for RepresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::update);
    }
}

impl RepresentationPlugin {
    fn update(reps: Query<(Entity, &Representation), With<NodeState>>, mut commands: Commands) {
        for (e, r) in &reps {
            let r = r.clone();
            commands.add(move |world: &mut World| {
                r.material.update(e, world);
            });
        }
    }
}
