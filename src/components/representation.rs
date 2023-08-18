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
    pub fn get_vec2_or_default(
        &self,
        var: VarName,
        default: Vec2,
        owner: Entity,
        world: &World,
    ) -> Vec2 {
        let value = self.0.get(&var).and_then(|x| x.get_vec2(owner, world).ok());
        value.unwrap_or(default)
    }
    pub fn get_float_or_default(
        &self,
        var: VarName,
        default: f32,
        owner: Entity,
        world: &World,
    ) -> f32 {
        let value = self
            .0
            .get(&var)
            .and_then(|x| x.get_float(owner, world).ok());
        value.unwrap_or(default)
    }
    pub fn get_string_or_default(
        &self,
        var: VarName,
        default: String,
        owner: Entity,
        world: &World,
    ) -> String {
        let value = self
            .0
            .get(&var)
            .and_then(|x| x.get_string(owner, world).ok());
        value.unwrap_or(default)
    }

    pub fn get_vec2(&self, var: VarName, owner: Entity, world: &World) -> Result<Vec2> {
        self.0
            .get(&var)
            .context("No mapping")
            .and_then(|x| x.get_vec2(owner, world))
    }

    pub fn get_float(&self, var: VarName, owner: Entity, world: &World) -> Result<f32> {
        self.0
            .get(&var)
            .context("No mapping")
            .and_then(|x| x.get_float(owner, world))
    }

    pub fn iter<'a>(&'a self) -> impl Iterator<Item = (&'a VarName, &'a Expression)> {
        self.0.iter()
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(tag = "type")]
#[serde(deny_unknown_fields)]
pub enum RepresentationMaterial {
    Rectangle {
        #[serde(default = "default_one_vec2")]
        size: Vec2,
        #[serde(default)]
        color: HexColor,
    },
    Circle {
        #[serde(default = "default_one_f32")]
        radius: f32,
        #[serde(default)]
        color: HexColor,
    },
    Text {
        #[serde(default = "default_one_f32")]
        size: f32,
        #[serde(default)]
        text: String,
        #[serde(default)]
        color: HexColor,
        #[serde(default = "default_font_size")]
        font_size: f32,
    },
}

fn default_font_size() -> f32 {
    32.0
}
fn default_one_f32() -> f32 {
    1.0
}
fn default_one_vec2() -> Vec2 {
    Vec2::ONE
}

impl RepresentationMaterial {
    pub fn get_mesh(&self, size: Vec2) -> Mesh {
        match self {
            RepresentationMaterial::Rectangle { .. } => Mesh::from(shape::Quad::new(size)),
            RepresentationMaterial::Circle { .. } => Mesh::from(shape::Circle::new(size.x)),
            _ => panic!("Can't generate mesh for {self:?}"),
        }
    }
    pub fn get_shape(&self) -> Shape {
        match self {
            RepresentationMaterial::Rectangle { .. } => Shape::Rectangle,
            RepresentationMaterial::Circle { .. } => Shape::Circle,
            _ => panic!("Can't generate shape for {self:?}"),
        }
    }
    pub fn get_size(&self) -> Vec2 {
        match self {
            RepresentationMaterial::Rectangle { size, .. } => *size,
            RepresentationMaterial::Circle { radius, .. } => vec2(*radius, *radius),
            RepresentationMaterial::Text {
                font_size: size, ..
            } => vec2(*size, *size),
        }
    }
}

impl Representation {
    pub fn unpack(mut self, parent: Option<Entity>, world: &mut World) -> Entity {
        let mut entity = match &self.material {
            RepresentationMaterial::Rectangle { color, .. }
            | RepresentationMaterial::Circle { color, .. } => {
                let mut materials = world.resource_mut::<Assets<SdfShapeMaterial>>();
                let material = materials.add(SdfShapeMaterial {
                    color: color.clone().into(),
                    shape: self.material.get_shape(),
                    ..default()
                });
                let mesh = world
                    .resource_mut::<Assets<Mesh>>()
                    .add(self.material.get_mesh(self.material.get_size()))
                    .into();
                world.spawn(MaterialMesh2dBundle {
                    mesh,
                    transform: Transform::default(),
                    material,
                    ..default()
                })
            }
            RepresentationMaterial::Text {
                text,
                color,
                font_size,
                size,
            } => world.spawn(Text2dBundle {
                text: Text::from_section(
                    text,
                    TextStyle {
                        font_size: *font_size,
                        color: color.clone().into(),
                        ..default()
                    },
                ),
                transform: Transform::from_scale(
                    vec3(1.0 / *font_size, 1.0 / *font_size, 1.0) * *size,
                ),
                ..default()
            }),
        };
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
