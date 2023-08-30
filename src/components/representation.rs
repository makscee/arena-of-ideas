use bevy::sprite::Mesh2dHandle;

use super::*;

#[derive(Serialize, TypeUuid, TypePath, Deserialize, Debug, Component, Resource, Clone)]
#[uuid = "cc360991-638e-4066-af03-f4f8abbbc450"]
#[serde(deny_unknown_fields)]
pub struct Representation {
    pub material: RepresentationMaterial,
    #[serde(default)]
    pub children: Vec<Box<Representation>>,
    #[serde(default)]
    pub mapping: VarMapping,
}

#[derive(Serialize, Deserialize, Debug, Component, Clone, Default)]
pub struct VarMapping(HashMap<VarName, Expression>);

impl VarMapping {
    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a VarName, &'a Expression)> {
        self.0.iter()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum RepresentationMaterial {
    Shape {
        shape: Shape,
        #[serde(default = "default_one_vec2")]
        size: Expression,
        #[serde(default)]
        color: HexColor,
    },
    Text {
        #[serde(default = "default_one_f32")]
        size: Expression,
        text: Expression,
        #[serde(default)]
        color: HexColor,
        #[serde(default = "default_font_size")]
        font_size: f32,
    },
}

fn default_font_size() -> f32 {
    32.0
}
fn default_one_f32() -> Expression {
    Expression::Float(1.0)
}
fn default_one_vec2() -> Expression {
    Expression::Vec2(1.0, 1.0)
}

impl RepresentationMaterial {
    pub fn unpack(&self, entity: Entity, world: &mut World) {
        match self {
            RepresentationMaterial::Shape { shape, color, .. } => {
                let mut materials = world.resource_mut::<Assets<LineShapeMaterial>>();
                let material = LineShapeMaterial {
                    color: color.clone().into(),
                    shape: *shape,
                    ..default()
                };
                let material = materials.add(material);
                let mesh = world
                    .resource_mut::<Assets<Mesh>>()
                    .add(Mesh::new(default()));
                world.entity_mut(entity).insert(MaterialMesh2dBundle {
                    material,
                    mesh: mesh.into(),
                    ..default()
                });
            }
            RepresentationMaterial::Text {
                color, font_size, ..
            } => {
                world.entity_mut(entity).insert(Text2dBundle {
                    text: Text::from_section(
                        "".to_owned(),
                        TextStyle {
                            font_size: *font_size,
                            color: color.clone().into(),
                            ..default()
                        },
                    ),
                    ..default()
                });
            }
        }
    }

    fn set_visible(entity: Entity, value: bool, world: &mut World) {
        if let Some(mut entity) = world.get_entity_mut(entity) {
            match value {
                true => entity.insert(Visibility::Inherited),
                false => entity.insert(Visibility::Hidden),
            };
        }
    }

    pub fn update(&self, entity: Entity, world: &mut World) {
        let t = world.get_resource::<GameTimer>().unwrap().get_t();
        if let Some(state) = world.get::<VarState>(entity) {
            let visible = state.get_bool_at(VarName::Visible, t).unwrap_or(true);
            Self::set_visible(entity, visible, world);
            if !visible {
                return;
            }
        }
        match self {
            RepresentationMaterial::Shape { shape, size, color } => {
                let size = size.get_vec2(entity, world).unwrap();
                let handle = world
                    .get::<Handle<LineShapeMaterial>>(entity)
                    .unwrap()
                    .clone();
                let mut materials = world
                    .get_resource_mut::<Assets<LineShapeMaterial>>()
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
                            *mesh = shape.mesh(size);
                        }
                    }
                }
            }
            RepresentationMaterial::Text {
                size,
                text,
                font_size,
                ..
            } => {
                world.get_mut::<Text>(entity).unwrap().sections[0].value =
                    text.get_string(entity, world).unwrap();
                world.get_mut::<Transform>(entity).unwrap().scale =
                    vec3(1.0 / *font_size, 1.0 / *font_size, 1.0)
                        * size.get_float(entity, world).unwrap();
            }
        }
    }
}

impl Representation {
    pub fn unpack(mut self, parent: Option<Entity>, world: &mut World) -> Entity {
        let entity = world.spawn_empty().id();
        self.material.unpack(entity, world);
        let mut entity = world.entity_mut(entity);
        entity.get_mut::<Transform>().unwrap().translation.z += 0.0000001; // children always rendered on top of parents
        if let Some(parent) = parent {
            entity.set_parent(parent);
        }
        let entity = entity.id();
        for (i, child) in self.children.drain(..).enumerate() {
            let entity = child.unpack(Some(entity), world);
            world.get_mut::<Transform>(entity).unwrap().translation.z += 0.001 * i as f32;
        }
        world.entity_mut(entity).insert(self);
        entity
    }
}
