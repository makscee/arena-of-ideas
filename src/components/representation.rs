use bevy::{
    prelude::{CubicBezier, CubicGenerator},
    sprite::Mesh2dHandle,
    transform::TransformBundle,
};
use bevy_egui::egui::{ComboBox, DragValue};
use indexmap::IndexMap;
use ron::ser::{to_string_pretty, PrettyConfig};

use super::*;

#[derive(
    Asset, Serialize, TypePath, Deserialize, Debug, Component, Resource, Clone, Default, PartialEq,
)]
#[serde(deny_unknown_fields)]
pub struct Representation {
    pub material: RepresentationMaterial,
    #[serde(default)]
    pub children: Vec<Box<Representation>>,
    #[serde(default)]
    pub mapping: IndexMap<VarName, Expression>,
    #[serde(default)]
    pub count: usize,
    #[serde(skip)]
    pub entity: Option<Entity>,
    #[serde(skip)]
    pub material_entities: Vec<Entity>,
}

#[derive(Resource)]
pub struct SkipVisual(pub bool);

impl SkipVisual {
    pub fn active(world: &mut World) -> bool {
        world
            .get_resource::<SkipVisual>()
            .map(|s| s.0)
            .unwrap_or_default()
    }

    pub fn set_active(value: bool, world: &mut World) {
        world.insert_resource(SkipVisual(value));
    }
}

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
    Expression::Int(1)
}
fn f32_one_e() -> Expression {
    Expression::Float(1.0)
}
fn f32_zero_e() -> Expression {
    Expression::Float(0.0)
}
fn f32_arr_e() -> Vec<Expression> {
    [Expression::Float(0.0), Expression::Float(1.0)].into()
}
fn vec2_zero_e() -> Expression {
    Expression::Vec2(0.0, 0.0)
}
fn vec2_one_e() -> Expression {
    Expression::Vec2(1.0, 1.0)
}
fn color_e() -> Expression {
    Expression::OwnerState(VarName::Color)
}
fn color_arr_e() -> Vec<Expression> {
    [
        Expression::OwnerState(VarName::Color),
        Expression::Hex("#ffffff".to_owned()),
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
    pub offset: Expression,
}

impl Default for RepShape {
    fn default() -> Self {
        Self::Circle {
            radius: Expression::Float(1.0),
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
                world.entity_mut(entity).insert((
                    Transform::default(),
                    GlobalTransform::default(),
                    VisibilityBundle::default(),
                ));
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

    fn set_visible(entity: Entity, value: bool, world: &mut World) {
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
                    }) = fbm
                    {
                        let octaves = octaves.get_int(context, world).unwrap_or(1);
                        let lacunarity = lacunarity.get_float(context, world).unwrap_or(1.0);
                        let gain = gain.get_float(context, world).unwrap_or(1.0);
                        let offset = offset.get_float(context, world).unwrap_or(1.0);
                        material.data[9].x = octaves as f32;
                        material.data[9].y = lacunarity;
                        material.data[9].z = gain;
                        material.data[9].w = offset;
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
                    .get_var(VarName::Delta, world)
                    .and_then(|x| x.get_vec2().ok())
                    .unwrap_or(vec2(1.0, 0.0));
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

    fn show_editor(&mut self, context: &Context, ui: &mut Ui, world: &mut World) {
        CollapsingHeader::new("Material")
            .default_open(true)
            .show(ui, |ui| {
                ComboBox::from_label("type")
                    .selected_text(self.to_string())
                    .show_ui(ui, |ui| {
                        for option in RepresentationMaterial::iter() {
                            let text = option.to_string();
                            if ui.selectable_value(self, option, text).changed() {
                                *self = ron::from_str(&if matches!(self, Self::None) {
                                    self.to_string()
                                } else {
                                    format!("{}()", self.to_string())
                                })
                                .unwrap();
                            }
                        }
                    });
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
                        ui.horizontal(|ui| {
                            ComboBox::from_label("shape")
                                .selected_text(shape.to_string())
                                .show_ui(ui, |ui| {
                                    for option in RepShape::iter() {
                                        let text = option.to_string();
                                        if ui.selectable_value(shape, option, text).changed() {
                                            *shape =
                                                ron::from_str(&format!("{}()", shape.to_string()))
                                                    .unwrap();
                                        }
                                    }
                                });
                            ComboBox::from_label("type")
                                .selected_text(shape_type.to_string())
                                .show_ui(ui, |ui| {
                                    for option in RepShapeType::iter() {
                                        let text = option.to_string();
                                        if ui.selectable_value(shape_type, option, text).changed() {
                                            let str = &match &shape_type {
                                                RepShapeType::Opaque => shape_type.to_string(),
                                                RepShapeType::Line { .. } => {
                                                    format!("{}()", shape_type.to_string())
                                                }
                                            };
                                            *shape_type = ron::from_str(str).unwrap();
                                        }
                                    }
                                });
                            ComboBox::from_label("fill")
                                .selected_text(fill.to_string())
                                .show_ui(ui, |ui| {
                                    for option in RepFill::iter() {
                                        let text = option.to_string();
                                        if ui.selectable_value(fill, option, text).changed() {
                                            *fill =
                                                ron::from_str(&format!("{}()", fill.to_string()))
                                                    .unwrap();
                                        }
                                    }
                                });
                            let mut fbm_enabled = fbm.is_some();
                            if ui.checkbox(&mut fbm_enabled, "fbm").changed() {
                                *fbm = if fbm_enabled {
                                    Some(ron::from_str("()").unwrap())
                                } else {
                                    None
                                };
                            }
                        });
                        show_tree("padding:", padding, context, ui, world);
                        show_tree("alpha:", alpha, context, ui, world);
                        match shape {
                            RepShape::Circle { radius } => {
                                show_tree("radius:", radius, context, ui, world)
                            }
                            RepShape::Rectangle { size } => {
                                show_tree("size:", size, context, ui, world)
                            }
                        }
                        match shape_type {
                            RepShapeType::Line { thickness } => {
                                show_tree("thickness:", thickness, context, ui, world)
                            }
                            RepShapeType::Opaque => {}
                        }
                        match fill {
                            RepFill::Solid { color } => {
                                show_tree("color:", color, context, ui, world)
                            }
                            RepFill::GradientLinear {
                                point1,
                                point2,
                                parts: _,
                                colors: _,
                            } => {
                                show_tree("point1:", point1, context, ui, world);
                                show_tree("point2:", point2, context, ui, world);
                            }
                            RepFill::GradientRadial {
                                center,
                                radius,
                                parts: _,
                                colors: _,
                            } => {
                                show_tree("center", center, context, ui, world);
                                show_tree("radius", radius, context, ui, world);
                            }
                        }
                        match fill {
                            RepFill::GradientLinear { parts, colors, .. }
                            | RepFill::GradientRadial { parts, colors, .. } => {
                                let mut delete = None;
                                for (i, color) in colors.into_iter().enumerate() {
                                    ui.horizontal(|ui| {
                                        if ui.button_red("-").clicked() {
                                            delete = Some(i);
                                        }
                                        show_trees(
                                            vec![
                                                (&format!("color{i}:"), color),
                                                (&format!("part{i}:"), &mut parts[i]),
                                            ],
                                            context,
                                            ui,
                                            world,
                                        );
                                    });
                                }
                                if ui.button("+").clicked() {
                                    parts.push(default());
                                    colors.push(default());
                                }
                                if let Some(i) = delete {
                                    parts.remove(i);
                                    colors.remove(i);
                                }
                            }
                            RepFill::Solid { .. } => {}
                        }

                        if let Some(RepFbm {
                            octaves,
                            lacunarity,
                            gain,
                            offset,
                        }) = fbm
                        {
                            show_tree("octaves:", octaves, context, ui, world);
                            show_tree("lacunarity:", lacunarity, context, ui, world);
                            show_tree("gain:", gain, context, ui, world);
                            show_tree("offset:", offset, context, ui, world);
                        }
                    }
                    RepresentationMaterial::Text {
                        size,
                        text,
                        color,
                        alpha,
                        font_size,
                    } => {
                        show_tree("size:", size, context, ui, world);
                        show_tree("text:", text, context, ui, world);
                        show_tree("color:", color, context, ui, world);
                        show_tree("alpha:", alpha, context, ui, world);
                        ui.label("font size:");
                        ui.add(Slider::new(font_size, 16.0..=48.0));
                    }
                    RepresentationMaterial::Curve {
                        thickness,
                        dilations: _,
                        curvature,
                        aa,
                        color,
                        alpha,
                    } => {
                        show_tree("thickness:", thickness, context, ui, world);
                        show_tree("curvature:", curvature, context, ui, world);
                        show_tree("aa:", aa, context, ui, world);
                        show_tree("alpha:", alpha, context, ui, world);
                        show_tree("color:", color, context, ui, world);
                    }
                };
            });
    }
}

impl Representation {
    pub fn unpack(mut self, entity: Entity, world: &mut World) -> Entity {
        let rep = self.unpack_materials(entity, world);
        world.entity_mut(entity).insert(self);
        rep
    }
    fn unpack_materials(&mut self, entity: Entity, world: &mut World) -> Entity {
        world
            .entity_mut(entity)
            .insert(TransformBundle::default())
            .insert(VisibilityBundle::default());
        VarState::default().attach(entity, world);
        self.entity = Some(entity);
        for i in 0..self.count.max(1) {
            let entity = world.spawn_empty().set_parent(entity).id();
            self.material.unpack(entity, world);
            VarState::new_with(VarName::Index, VarValue::Int(i as i32)).attach(entity, world);
            world.get_mut::<Transform>(entity).unwrap().translation.z += 0.001 * i as f32;
            self.material_entities.push(entity);
        }
        self.unpack_children(world);
        *self.material_entities.first().unwrap()
    }
    fn unpack_children(&mut self, world: &mut World) {
        let parent = *self.material_entities.first().unwrap();
        for (i, child) in self.children.iter_mut().enumerate() {
            let entity = world.spawn_empty().set_parent(parent).id();
            child.unpack_materials(entity, world);
            world.get_mut::<Transform>(entity).unwrap().translation.z += (i + 1) as f32;
        }
    }

    pub fn pack(entity: Entity, world: &World) -> Self {
        let mut rep = world.get::<Representation>(entity).unwrap().clone();
        rep.material_entities.clear();
        rep.entity = None;
        rep
    }
    pub fn update(self, dragged: Option<Entity>, world: &mut World) {
        let t = GameTimer::get().play_head();
        let entity = self.entity.unwrap();
        let context = Context::from_owner(entity, world);
        {
            let state = VarState::get_mut(entity, world);
            let visible = state.get_bool_at(VarName::Visible, t).unwrap_or(true);
            let visible = visible && state.birth < t;
            RepresentationMaterial::set_visible(entity, visible, world);
            if !visible {
                return;
            }
        }
        self.apply_mapping(entity, world);
        let vars: Vec<VarName> = [VarName::Position, VarName::Scale].into();
        VarState::apply_transform(entity, t, vars, world);
        for (i, entity) in self.material_entities.iter().enumerate() {
            let context = context
                .clone()
                .set_owner(*entity, world)
                .set_var(VarName::Index, VarValue::Int(i as i32))
                .take();
            self.apply_mapping(*entity, world);
            if dragged == Some(*entity) {
                debug!("Dragged");
            }
            VarState::apply_transform(
                *entity,
                t,
                [VarName::Rotation, VarName::Scale, VarName::Offset].into(),
                world,
            );
            self.material.update(
                *entity,
                &context
                    .clone()
                    .set_var(VarName::Index, VarValue::Int(i as i32)),
                world,
            );
        }
        for child in self.children {
            child.update(dragged, world);
        }
    }

    fn apply_mapping(&self, entity: Entity, world: &mut World) {
        let context = Context::from_owner(entity, world);
        let mapping: HashMap<VarName, VarValue> =
            HashMap::from_iter(self.mapping.iter().filter_map(|(var, value)| {
                match value.get_value(&context, world) {
                    Ok(value) => Some((*var, value)),
                    Err(_) => None,
                }
            }));
        let mut state = VarState::get_mut(entity, world);
        for (var, value) in mapping {
            state.init(var, value);
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

    pub fn show_editor(
        &mut self,
        context: &Context,
        id: impl std::hash::Hash,
        ui: &mut Ui,
        world: &mut World,
    ) {
        let response = CollapsingHeader::new("Representation")
            .id_source(&id)
            .show(ui, |ui| {
                frame(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("count:");
                        DragValue::new(&mut self.count).ui(ui);
                    });
                    self.material.show_editor(context, ui, world);

                    ui.horizontal(|ui| {
                        ui.label("mapping:");
                        if ui.button("+").clicked() {
                            self.mapping.shift_remove(&VarName::None);
                            self.mapping.insert(VarName::None, default());
                        }
                    });
                    let mut map_key = None;
                    let mut to_remove = None;
                    for (key, value) in self.mapping.iter_mut() {
                        let mut new_key = key.clone();
                        ui.horizontal(|ui| {
                            if ui.button_red("-").clicked() {
                                to_remove = Some(*key);
                            }
                            ui.collapsing(key.to_string(), |ui| {
                                frame(ui, |ui| {
                                    new_key.show_editor(Id::new(&id).with(*key), ui);
                                    show_tree("", value, context, ui, world);
                                });
                            });
                        });
                        if !new_key.eq(key) {
                            map_key = Some((*key, new_key));
                        }
                    }
                    if let Some((from, to)) = map_key {
                        let ind = self.mapping.get_index_of(&from).unwrap();
                        self.mapping.shift_remove(&to);
                        let value = self.mapping.shift_remove(&from).unwrap();
                        self.mapping.shift_insert(ind, to, value);
                    }
                    if let Some(key) = to_remove {
                        self.mapping.shift_remove(&key);
                    }
                    ui.horizontal(|ui| {
                        ui.label("children:");
                        if ui.button("+").clicked() {
                            self.children.push(default());
                        }
                    });
                    let mut to_remove = None;
                    for (i, child) in self.children.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            if ui.button_red("-").clicked() {
                                to_remove = Some(i);
                            }
                            child.show_editor(context, i, ui, world);
                        });
                    }
                    if let Some(i) = to_remove {
                        self.children.remove(i);
                    }
                });
            });
        response.header_response.context_menu(|ui| {
            if ui.button("Copy").clicked() {
                save_to_clipboard(&to_string_pretty(self, PrettyConfig::new()).unwrap(), world);
                ui.close_menu();
            }
            if ui.button("Paste").clicked() {
                if let Some(s) = get_from_clipboard(world) {
                    match ron::from_str(&s) {
                        Ok(o) => *self = o,
                        Err(e) => AlertPlugin::add_error(
                            Some("Paste Failed".to_owned()),
                            e.to_string(),
                            None,
                        ),
                    }
                }
                ui.close_menu();
            }
        });
    }
}
