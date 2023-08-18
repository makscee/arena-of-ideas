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
            let mut size = rep.material.get_size();
            for (key, value) in rep.mapping.iter() {
                match key {
                    VarName::Position => {
                        let value = value.get_vec2(entity, world).unwrap();
                        let mut transform = world.get_mut::<Transform>(entity).unwrap();
                        transform.translation.x = value.x;
                        transform.translation.y = value.y;
                    }
                    VarName::Size => {
                        let value = value.get_vec2(entity, world).unwrap();
                        size = value;
                    }
                    VarName::Radius => {
                        let value = value.get_float(entity, world).unwrap();
                        size = vec2(value, value);
                    }
                    _ => continue,
                };
            }

            match &rep.material {
                RepresentationMaterial::Rectangle { color, .. }
                | RepresentationMaterial::Circle { color, .. } => {
                    let handle = world
                        .get::<Handle<SdfShapeMaterial>>(entity)
                        .unwrap()
                        .clone();
                    let mut materials = world
                        .get_resource_mut::<Assets<SdfShapeMaterial>>()
                        .unwrap();
                    if let Some(mat) = materials.get_mut(&handle) {
                        mat.color = color.clone().into();
                        if mat.size != size {
                            mat.size = size;
                            let mesh = world.entity(entity).get::<Mesh2dHandle>().unwrap().clone();
                            if let Some(mesh) = world
                                .get_resource_mut::<Assets<Mesh>>()
                                .unwrap()
                                .get_mut(&mesh.0)
                            {
                                *mesh = rep.material.get_mesh(size);
                            }
                        }
                    }
                }
                RepresentationMaterial::Text {
                    text,
                    color,
                    size,
                    font_size,
                } => {
                    let text = rep.mapping.get_string_or_default(
                        VarName::Text,
                        text.into(),
                        entity,
                        world,
                    );
                    world.get_mut::<Text>(entity).unwrap().sections[0].value = text;
                }
            }
        }
    }
}
