use bevy::sprite::Mesh2dHandle;

use super::*;

pub struct RepresentationPlugin;

impl Plugin for RepresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, Self::injector_system);
    }
}

impl RepresentationPlugin {
    fn injector_system(world: &mut World) {
        let reps = world
            .query::<(Entity, &Representation)>()
            .iter(world)
            .map(|(e, r)| (e, r.clone()))
            .collect_vec();
        for (entity, rep) in reps {
            for (key, value) in rep.mapping.iter() {
                match key {
                    VarName::Position => {
                        let value = value.get_vec2(entity, world).unwrap();
                        let mut transform = world.get_mut::<Transform>(entity).unwrap();
                        transform.translation.x = value.x;
                        transform.translation.y = value.y;
                    }
                    _ => continue,
                };
            }

            rep.material.update(entity, world);
        }
    }
}
