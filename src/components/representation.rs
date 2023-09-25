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
        #[serde(default = "default_one_vec2_e")]
        size: Expression,
        #[serde(default = "default_color_e")]
        color: Expression,
    },
    Text {
        #[serde(default = "default_one_f32_e")]
        size: Expression,
        text: Expression,
        #[serde(default = "default_color_e")]
        color: Expression,
        #[serde(default = "default_font_size")]
        font_size: f32,
    },
}

fn default_font_size() -> f32 {
    32.0
}
fn default_one_f32_e() -> Expression {
    Expression::Float(1.0)
}
fn default_one_vec2_e() -> Expression {
    Expression::Vec2(1.0, 1.0)
}
fn default_color_e() -> Expression {
    Expression::Hex("#ff00ff".to_owned())
}

impl RepresentationMaterial {
    pub fn unpack(&self, entity: Entity, world: &mut World) {
        match self {
            RepresentationMaterial::Shape { shape, .. } => {
                let mut materials = world.resource_mut::<Assets<LineShapeMaterial>>();
                let material = LineShapeMaterial {
                    color: Color::PINK,
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
            RepresentationMaterial::Text { font_size, .. } => {
                world.entity_mut(entity).insert(Text2dBundle {
                    text: Text::from_section(
                        "".to_owned(),
                        TextStyle {
                            font_size: *font_size,
                            color: Color::PINK,
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
        let t = get_t(world);
        if let Some(state) = world.get::<VarState>(entity) {
            let visible = state.get_bool_at(VarName::Visible, t).unwrap_or(true);
            let visible = visible && state.birth < t;
            Self::set_visible(entity, visible, world);
            if !visible {
                return;
            }
        }
        let context = Context::from_owner(entity, world);
        match self {
            RepresentationMaterial::Shape { shape, size, color } => {
                let size = size.get_vec2(&context, world).unwrap_or_default();
                let color = color.get_color(&context, world).unwrap();
                let handle = world
                    .get::<Handle<LineShapeMaterial>>(entity)
                    .unwrap()
                    .clone();
                let mut materials = world
                    .get_resource_mut::<Assets<LineShapeMaterial>>()
                    .unwrap();
                if let Some(mat) = materials.get_mut(&handle) {
                    mat.color = color;
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
                color,
            } => {
                let color = color.get_color(&context, world).unwrap();
                world.get_mut::<Text>(entity).unwrap().sections[0].value =
                    text.get_string(&context, world).unwrap_or_default();
                world.get_mut::<Text>(entity).unwrap().sections[0].style = TextStyle {
                    font_size: *font_size,
                    color,
                    ..default()
                };
                world.get_mut::<Transform>(entity).unwrap().scale =
                    vec3(1.0 / *font_size, 1.0 / *font_size, 1.0)
                        * size.get_float(&context, world).unwrap();
            }
        }
    }
}

impl Representation {
    pub fn unpack(
        mut self,
        entity: Option<Entity>,
        parent: Option<Entity>,
        world: &mut World,
    ) -> Entity {
        let entity = match entity {
            Some(value) => value,
            None => world.spawn_empty().id(),
        };
        self.material.unpack(entity, world);
        let mut entity = world.entity_mut(entity);
        entity.get_mut::<Transform>().unwrap().translation.z += 0.0000001; // children always rendered on top of parents
        if let Some(parent) = parent {
            entity.set_parent(parent);
        }
        let entity = entity.id();
        for (i, child) in self.children.drain(..).enumerate() {
            let entity = child.unpack(None, Some(entity), world);
            world.get_mut::<Transform>(entity).unwrap().translation.z += 0.001 * i as f32;
        }
        world.entity_mut(entity).insert(self);
        entity
    }
    pub fn pack(entity: Entity, world: &World) -> Self {
        let mut rep = world.get::<Representation>(entity).unwrap().clone();
        rep.pack_children(entity, world);
        rep
    }
    fn pack_children(&mut self, entity: Entity, world: &World) {
        if let Some(children) = world.get::<Children>(entity) {
            for child in children.iter() {
                if let Some(mut rep) = world.get::<Representation>(*child).cloned() {
                    rep.pack_children(*child, world);
                    self.children.push(Box::new(rep));
                }
            }
        }
    }
}
