use bevy::{
    prelude::{Bezier, CubicGenerator},
    sprite::Mesh2dHandle,
};

use super::*;

#[derive(Serialize, TypeUuid, TypePath, Deserialize, Debug, Component, Resource, Clone)]
#[uuid = "cc360991-638e-4066-af03-f4f8abbbc450"]
#[serde(deny_unknown_fields)]
pub struct Representation {
    pub material: RepresentationMaterial,
    #[serde(default)]
    pub children: Vec<Box<Representation>>,
    #[serde(default)]
    pub mapping: HashMap<VarName, Expression>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub enum RepresentationMaterial {
    Repeat {
        count: Expression,
        source: Box<Representation>,
    },
    Shape {
        #[serde(default)]
        shape: Shape,
        #[serde(default)]
        fill: Fill,
        #[serde(default = "default_one_vec2_e")]
        size: Expression,
        #[serde(default = "default_one_f32_e")]
        thickness: Expression,
        #[serde(default = "default_one_f32_e")]
        alpha: Expression,
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
    Curve {
        #[serde(default = "default_one_f32_e")]
        thickness: Expression,
        #[serde(default)]
        dilations: Vec<(Expression, Expression)>,
        #[serde(default = "default_one_f32_e")]
        curvature: Expression,
        #[serde(default = "default_zero_f32_e")]
        aa: Expression,
        #[serde(default = "default_color_e")]
        color: Expression,
    },
}

fn default_font_size() -> f32 {
    32.0
}
fn default_one_f32_e() -> Expression {
    Expression::Float(1.0)
}
fn default_zero_f32_e() -> Expression {
    Expression::Float(0.0)
}
fn default_one_vec2_e() -> Expression {
    Expression::Vec2(1.0, 1.0)
}
fn default_color_e() -> Expression {
    Expression::State(VarName::HouseColor)
}

impl RepresentationMaterial {
    pub fn unpack(&self, entity: Entity, world: &mut World) {
        match self {
            RepresentationMaterial::Shape { shape, fill, .. } => {
                let mut materials = world.resource_mut::<Assets<ShapeMaterial>>();
                let material = ShapeMaterial {
                    color: Color::PINK,
                    shape: *shape,
                    fill: *fill,
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
            RepresentationMaterial::Curve { .. } => {
                let mut materials = world.resource_mut::<Assets<CurveMaterial>>();
                let material = CurveMaterial {
                    color: Color::PINK,
                    ..default()
                };
                let material = materials.add(material);
                let mesh = world
                    .resource_mut::<Assets<Mesh>>()
                    .add(Mesh::new(PrimitiveTopology::TriangleStrip));
                world.entity_mut(entity).insert(MaterialMesh2dBundle {
                    material,
                    mesh: mesh.into(),
                    ..default()
                });
            }
            RepresentationMaterial::Repeat { count, source } => {
                for i in 0..count
                    .get_int(&Context::from_owner(entity, world), world)
                    .unwrap()
                {
                    let child = source.clone().unpack(None, Some(entity), world);
                    world.entity_mut(entity).insert((
                        Transform::default(),
                        GlobalTransform::default(),
                        VisibilityBundle::default(),
                    ));
                    VarState::get_mut(child, world).init(VarName::Index, VarValue::Int(i));
                }
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
        if let Ok(state) = VarState::try_find(entity, world) {
            let visible = state.get_bool_at(VarName::Visible, t).unwrap_or(true);
            let visible = visible && state.birth < t;
            Self::set_visible(entity, visible, world);
            if !visible {
                return;
            }
        }
        let context = Context::from_owner(entity, world);
        match self {
            RepresentationMaterial::Shape {
                shape,
                size,
                color,
                thickness,
                alpha,
                ..
            } => {
                let size = size.get_vec2(&context, world).unwrap_or_default();
                let thickness = thickness.get_float(&context, world).unwrap_or_default();
                let alpha = alpha.get_float(&context, world).unwrap_or_default();
                let color = color.get_color(&context, world).unwrap_or(Color::Rgba {
                    red: 1.0,
                    green: 0.0,
                    blue: 1.0,
                    alpha: 1.0,
                });
                let handle = world.get::<Handle<ShapeMaterial>>(entity).unwrap().clone();
                let mut materials = world.get_resource_mut::<Assets<ShapeMaterial>>().unwrap();
                if let Some(mat) = materials.get_mut(&handle) {
                    mat.color = color;
                    mat.thickness = thickness;
                    mat.alpha = alpha;
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
            RepresentationMaterial::Curve {
                thickness,
                curvature,
                color,
                dilations,
                aa,
            } => {
                let thickness = thickness.get_float(&context, world).unwrap() * 0.05;
                let curvature = curvature.get_float(&context, world).unwrap();
                let aa = aa.get_float(&context, world).unwrap();
                let color = color.get_color(&context, world).unwrap();
                let mut dilations = dilations
                    .into_iter()
                    .map(|(t, v)| {
                        (
                            t.get_float(&context, world).unwrap(),
                            v.get_float(&context, world).unwrap(),
                        )
                    })
                    .sorted_by(|a, b| a.0.total_cmp(&b.0))
                    .collect_vec();
                if dilations.get(0).is_none() || dilations[0].0 != 0.0 {
                    dilations.insert(0, (0.0, 0.0));
                }
                if dilations.last().unwrap().0 != 1.0 {
                    dilations.push((1.0, dilations.last().unwrap().1));
                }

                let delta = context
                    .get_var(VarName::Delta, world)
                    .unwrap()
                    .get_vec2()
                    .unwrap();
                let control_delta = vec2(0.0, curvature);
                let curve =
                    Bezier::new([[Vec2::ZERO, control_delta, delta + control_delta, delta]])
                        .to_curve();
                let mut points: Vec<Vec3> = default();
                let mut uvs: Vec<Vec2> = default();
                const SEGMENTS: usize = 30;
                for t in 0..SEGMENTS {
                    let t = t as f32 / SEGMENTS as f32;
                    let position = curve.position(t).extend(0.0);
                    let velocity = curve.velocity(t);
                    let mut dilation = 0.0;
                    for ind in 0..dilations.len() - 1 {
                        let (p1, v1) = dilations[ind];
                        let (p2, v2) = dilations[ind + 1];
                        if p1 <= t && p2 >= t {
                            dilation = v1 + (t - p1) / (p2 - p1) * (v2 - v1);
                        }
                    }
                    points.push(
                        position
                            + (Vec2::NEG_Y.rotate(velocity.normalize())
                                * thickness
                                * (1.0 + dilation))
                                .extend(0.0),
                    );
                    points.push(
                        position
                            + (Vec2::Y.rotate(velocity.normalize()) * thickness * (1.0 + dilation))
                                .extend(0.0),
                    );
                    uvs.push(vec2(t, -1.0));
                    uvs.push(vec2(t, 1.0));
                }

                let handle = world.get::<Handle<CurveMaterial>>(entity).unwrap().clone();
                let mut materials = world.get_resource_mut::<Assets<CurveMaterial>>().unwrap();
                if let Some(mat) = materials.get_mut(&handle) {
                    mat.color = color;
                    mat.aa = aa;
                    let mesh = world.entity(entity).get::<Mesh2dHandle>().unwrap().clone();
                    if let Some(mesh) = world
                        .get_resource_mut::<Assets<Mesh>>()
                        .unwrap()
                        .get_mut(&mesh.0)
                    {
                        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, points);
                        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
                    }
                }
            }
            RepresentationMaterial::Repeat { .. } => {}
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
        if !world.entity(entity).contains::<VarState>() {
            VarState::default().attach(entity, world);
        }
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

    pub fn despawn_all(world: &mut World) {
        for entity in world
            .query_filtered::<Entity, With<Representation>>()
            .iter(world)
            .collect_vec()
        {
            if let Some(entity) = world.get_entity_mut(entity) {
                entity.despawn_recursive()
            }
        }
    }
}
