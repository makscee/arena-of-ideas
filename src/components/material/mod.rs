pub use super::*;

mod curve;
mod shape;

pub use curve::*;
pub use shape::*;

#[derive(Serialize, Deserialize, Debug, Clone, Display, Default, EnumIter, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum RepresentationMaterial {
    #[default]
    None,
    Shape {
        #[serde(default)]
        shape: RepShape,
        #[serde(default)]
        shape_type: RepShapeType,
        #[serde(default)]
        fill: RepFill,
        #[serde(default)]
        fbm: Option<RepFbm>,
        #[serde(default = "f32_one_e")]
        alpha: Expression,
        #[serde(default = "f32_zero_e")]
        padding: Expression,
    },
    Text {
        #[serde(default = "f32_one_e")]
        size: Expression,
        text: Expression,
        #[serde(default = "color_e")]
        color: Expression,
        #[serde(default = "f32_one_e")]
        alpha: Expression,
        #[serde(default = "font_size")]
        font_size: f32,
    },
    Curve {
        #[serde(default = "f32_one_e")]
        thickness: Expression,
        #[serde(default)]
        dilations: Vec<(Expression, Expression)>,
        #[serde(default = "f32_one_e")]
        curvature: Expression,
        #[serde(default = "f32_zero_e")]
        aa: Expression,
        #[serde(default = "f32_one_e")]
        alpha: Expression,
        #[serde(default = "color_e")]
        color: Expression,
    },
}

fn font_size() -> f32 {
    32.0
}
fn i32_one_e() -> Expression {
    Expression::Value(VarValue::Int(1))
}
fn f32_one_e() -> Expression {
    Expression::Value(VarValue::Float(1.0))
}
fn f32_zero_e() -> Expression {
    Expression::Value(VarValue::Float(0.0))
}
fn f32_arr_e() -> Vec<Expression> {
    [
        Expression::Value(VarValue::Float(0.0)),
        Expression::Value(VarValue::Float(1.0)),
    ]
    .into()
}
fn vec2_zero_e() -> Expression {
    Expression::Value(VarValue::Vec2(vec2(0.0, 0.0)))
}
fn vec2_one_e() -> Expression {
    Expression::Value(VarValue::Vec2(vec2(1.0, 1.0)))
}
fn color_e() -> Expression {
    Expression::OwnerState(VarName::Color)
}
fn color_arr_e() -> Vec<Expression> {
    [
        Expression::OwnerState(VarName::Color),
        Expression::HexColor("#ffffff".to_owned()),
    ]
    .into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter, Display)]
pub enum RepShape {
    Circle {
        #[serde(default = "f32_one_e")]
        radius: Expression,
    },
    Rectangle {
        #[serde(default = "vec2_one_e")]
        size: Expression,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter, Display)]
pub enum RepFill {
    Solid {
        #[serde(default = "color_e")]
        color: Expression,
    },
    GradientLinear {
        #[serde(default = "vec2_zero_e")]
        point1: Expression,
        #[serde(default = "vec2_one_e")]
        point2: Expression,
        #[serde(default = "f32_arr_e")]
        parts: Vec<Expression>,
        #[serde(default = "color_arr_e")]
        colors: Vec<Expression>,
    },
    GradientRadial {
        #[serde(default = "vec2_zero_e")]
        center: Expression,
        #[serde(default = "f32_one_e")]
        radius: Expression,
        #[serde(default = "f32_arr_e")]
        parts: Vec<Expression>,
        #[serde(default = "color_arr_e")]
        colors: Vec<Expression>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, EnumIter, Display)]
pub enum RepShapeType {
    #[default]
    Opaque,
    Line {
        #[serde(default = "f32_one_e")]
        thickness: Expression,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RepFbm {
    #[serde(default = "i32_one_e")]
    pub octaves: Expression,
    #[serde(default = "f32_one_e")]
    pub lacunarity: Expression,
    #[serde(default = "f32_one_e")]
    pub gain: Expression,
    #[serde(default = "f32_one_e")]
    pub strength: Expression,
    #[serde(default = "vec2_one_e")]
    pub offset: Expression,
}

impl Default for RepShape {
    fn default() -> Self {
        Self::Circle {
            radius: Expression::Value(VarValue::Float(1.0)),
        }
    }
}

impl Default for RepFill {
    fn default() -> Self {
        Self::Solid {
            color: Expression::OwnerState(VarName::Color),
        }
    }
}

impl RepShape {
    fn shader_shape(&self) -> ShaderShape {
        match self {
            RepShape::Circle { .. } => ShaderShape::Circle,
            RepShape::Rectangle { .. } => ShaderShape::Rectangle,
        }
    }
}

impl RepFill {
    fn shader_fill(&self) -> ShaderShapeFill {
        match self {
            RepFill::Solid { .. } => ShaderShapeFill::Solid,
            RepFill::GradientLinear { .. } => ShaderShapeFill::GradientLinear,
            RepFill::GradientRadial { .. } => ShaderShapeFill::GradientRadial,
        }
    }
}

impl RepShapeType {
    fn shader_shape_type(&self) -> ShaderShapeType {
        match self {
            RepShapeType::Opaque => ShaderShapeType::Opaque,
            RepShapeType::Line { .. } => ShaderShapeType::Line,
        }
    }
}

impl RepresentationMaterial {
    pub fn unpack(&self, entity: Entity, world: &mut World) {
        match self {
            RepresentationMaterial::None => {
                world
                    .entity_mut(entity)
                    .insert((TransformBundle::default(), VisibilityBundle::default()));
            }
            RepresentationMaterial::Shape {
                shape,
                shape_type,
                fill,
                fbm,
                ..
            } => {
                let mut materials = world.resource_mut::<Assets<ShapeMaterial>>();
                let material = ShapeMaterial {
                    shape: shape.shader_shape(),
                    shape_type: shape_type.shader_shape_type(),
                    shape_fill: fill.shader_fill(),
                    fbm: fbm.is_some(),
                    ..default()
                };
                let material = materials.add(material);
                let mesh = world
                    .resource_mut::<Assets<Mesh>>()
                    .add(Mesh::new(default(), default()));
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
                        bevy::text::TextStyle {
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
                    .add(Mesh::new(PrimitiveTopology::TriangleStrip, default()));
                world.entity_mut(entity).insert(MaterialMesh2dBundle {
                    material,
                    mesh: mesh.into(),
                    ..default()
                });
            }
        }
    }

    pub fn set_visible(entity: Entity, value: bool, world: &mut World) {
        if let Some(mut entity) = world.get_entity_mut(entity) {
            match value {
                true => entity.insert(Visibility::Inherited),
                false => entity.insert(Visibility::Hidden),
            };
        }
    }

    pub fn update(&self, entity: Entity, context: &Context, world: &mut World) {
        match self {
            RepresentationMaterial::None => {}
            RepresentationMaterial::Shape {
                shape,
                shape_type,
                fill,
                alpha,
                padding,
                fbm,
            } => {
                let handle = world.get::<Handle<ShapeMaterial>>(entity).unwrap().clone();
                if let Some(mut material) = world
                    .get_resource_mut::<Assets<ShapeMaterial>>()
                    .unwrap()
                    .remove(&handle)
                {
                    let mut refresh_mesh = false;
                    let padding = padding.get_float(context, world).unwrap_or_default();
                    material.data[1].z = padding;
                    match shape {
                        RepShape::Circle { radius } => {
                            let radius = radius.get_float(context, world).unwrap_or(1.0);
                            let t = &mut material.data[10];
                            if radius != t.x {
                                refresh_mesh = true;
                            }
                            *t = vec4(radius, radius, 0.0, 0.0);
                        }
                        RepShape::Rectangle { size } => {
                            let size = size.get_vec2(context, world).unwrap_or(vec2(1.0, 1.0));
                            let t = &mut material.data[10];
                            if t.xy() != size {
                                refresh_mesh = true;
                            }
                            *t = Vec4::from((size, 0.0, 0.0));
                        }
                    }
                    match shape_type {
                        RepShapeType::Line { thickness } => {
                            material.data[10].w = thickness.get_float(context, world).unwrap_or(1.0)
                        }
                        RepShapeType::Opaque => {}
                    }
                    match fill {
                        RepFill::Solid { color } => {
                            material.colors[0] =
                                color.get_color(context, world).unwrap_or(Color::FUCHSIA)
                        }
                        RepFill::GradientLinear {
                            point1,
                            point2,
                            parts: _,
                            colors: _,
                        } => {
                            let point1 = point1.get_vec2(context, world).unwrap_or_default();
                            let point2 = point2.get_vec2(context, world).unwrap_or_default();
                            material.data[0].x = point1.x;
                            material.data[0].y = point1.y;
                            material.data[1].x = point2.x;
                            material.data[1].y = point2.y;
                        }
                        RepFill::GradientRadial {
                            center,
                            radius,
                            parts: _,
                            colors: _,
                        } => {
                            let center = center.get_vec2(context, world).unwrap_or_default();
                            material.data[0].x = center.x;
                            material.data[0].y = center.y;
                            let radius = radius.get_float(context, world).unwrap_or(1.0);
                            material.data[0].z = radius;
                        }
                    }
                    match fill {
                        RepFill::GradientLinear { parts, colors, .. }
                        | RepFill::GradientRadial { parts, colors, .. } => {
                            for (i, color) in colors.into_iter().enumerate() {
                                let color =
                                    color.get_color(context, world).unwrap_or(Color::FUCHSIA);
                                let part = parts[i].get_float(context, world).unwrap_or(0.5);
                                material.colors[i] = color;
                                material.data[i].w = part;
                            }
                        }
                        RepFill::Solid { .. } => {}
                    }
                    material.data[10].z = alpha.get_float(context, world).unwrap_or(1.0);

                    if let Some(RepFbm {
                        octaves,
                        lacunarity,
                        gain,
                        offset,
                        strength,
                    }) = fbm
                    {
                        let octaves = octaves.get_int(context, world).unwrap_or(1);
                        let lacunarity = lacunarity.get_float(context, world).unwrap_or(1.0);
                        let gain = gain.get_float(context, world).unwrap_or(1.0);
                        let strength = strength.get_float(context, world).unwrap_or(1.0);
                        let offset = offset.get_vec2(context, world).unwrap_or(Vec2::ONE);
                        material.data[9].x = octaves as f32;
                        material.data[9].y = lacunarity;
                        material.data[9].z = gain;
                        material.data[8].x = offset.x;
                        material.data[8].y = offset.y;
                        material.data[8].z = strength;
                    }
                    if refresh_mesh {
                        let mesh = world.entity(entity).get::<Mesh2dHandle>().unwrap().clone();
                        if let Some(mesh) = world
                            .get_resource_mut::<Assets<Mesh>>()
                            .unwrap()
                            .get_mut(&mesh.0)
                        {
                            let size = material.data[10].xy() + vec2(padding, padding);
                            if size.x > 0.0 && size.y >= 0.0 {
                                *mesh = shape.shader_shape().mesh(size);
                            }
                        }
                    }
                    let _ = world
                        .get_resource_mut::<Assets<ShapeMaterial>>()
                        .unwrap()
                        .insert(handle, material);
                }
            }
            RepresentationMaterial::Text {
                size,
                text,
                color,
                alpha,
                font_size,
            } => {
                let color = color
                    .get_color(context, world)
                    .unwrap_or_default()
                    .set_a(alpha.get_float(context, world).unwrap_or(1.0))
                    .to_owned();
                let text = text.get_string(context, world).unwrap_or_default();
                let text_comp = &mut world.get_mut::<Text>(entity).unwrap().sections[0];
                text_comp.value = text;
                text_comp.style = bevy::text::TextStyle {
                    font_size: *font_size,
                    color,
                    ..default()
                };
                world.get_mut::<Transform>(entity).unwrap().scale =
                    vec3(1.0 / *font_size, 1.0 / *font_size, 1.0)
                        * size.get_float(context, world).unwrap();
            }
            RepresentationMaterial::Curve {
                thickness,
                curvature,
                color,
                dilations,
                aa,
                alpha,
            } => {
                let thickness = thickness.get_float(context, world).unwrap_or(1.0) * 0.05;
                let curvature = curvature.get_float(context, world).unwrap_or(1.0);
                let aa = aa.get_float(context, world).unwrap_or(1.0);
                let color = color.get_color(context, world).unwrap_or_default();
                let alpha = alpha.get_float(context, world).unwrap_or(1.0);
                let mut dilations = dilations
                    .iter()
                    .map(|(t, v)| {
                        (
                            t.get_float(context, world).unwrap(),
                            v.get_float(context, world).unwrap(),
                        )
                    })
                    .sorted_by(|a, b| a.0.total_cmp(&b.0))
                    .collect_vec();
                if dilations.first().is_none() || dilations[0].0 != 0.0 {
                    dilations.insert(0, (0.0, 0.0));
                }
                if dilations.last().unwrap().0 != 1.0 {
                    dilations.push((1.0, dilations.last().unwrap().1));
                }

                let delta = context
                    .get_value(VarName::Delta, world)
                    .and_then(|x| x.get_vec2())
                    .unwrap_or(vec2(0.0, 0.0));
                let control_delta = vec2(0.0, curvature);
                let curve =
                    CubicBezier::new([[Vec2::ZERO, control_delta, delta + control_delta, delta]])
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
                    mat.alpha = alpha;
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
        }
    }
}
