use bevy::{
    prelude::{Bezier, CubicGenerator},
    sprite::Mesh2dHandle,
    transform::TransformBundle,
};
use bevy_egui::egui::{ComboBox, DragValue};
use indexmap::IndexMap;
use ron::ser::{to_string_pretty, PrettyConfig};

use super::*;

#[derive(
    Serialize,
    TypeUuid,
    TypePath,
    Deserialize,
    Debug,
    Component,
    Resource,
    Clone,
    Default,
    PartialEq,
)]
#[uuid = "cc360991-638e-4066-af03-f4f8abbbc450"]
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
        #[serde(default = "default_one_f32_e")]
        alpha: Expression,
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
    Expression::State(VarName::Color)
}

impl RepresentationMaterial {
    pub fn fill_default(&mut self) -> &mut Self {
        match self {
            RepresentationMaterial::None => {}
            RepresentationMaterial::Shape {
                size,
                thickness,
                alpha,
                color,
                ..
            } => {
                *size = default_one_vec2_e();
                *thickness = default_one_f32_e();
                *alpha = default_one_f32_e();
                *color = default_color_e();
            }
            RepresentationMaterial::Text {
                size,
                text,
                color,
                font_size,
                ..
            } => {
                *size = default_one_f32_e();
                *color = default_color_e();
                *font_size = 16.0;
                *text = Expression::String("empty".to_owned());
            }
            RepresentationMaterial::Curve {
                thickness,
                curvature,
                color,
                ..
            } => {
                *thickness = default_one_f32_e();
                *color = default_color_e();
                *curvature = default_one_f32_e();
            }
        }
        self
    }

    pub fn unpack(&self, entity: Entity, world: &mut World) {
        match self {
            RepresentationMaterial::None => {
                world.entity_mut(entity).insert((
                    Transform::default(),
                    GlobalTransform::default(),
                    VisibilityBundle::default(),
                ));
            }
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
                    .add(Mesh::new(PrimitiveTopology::TriangleStrip));
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
                size,
                color,
                thickness,
                alpha,
                ..
            } => {
                let size = size.get_vec2(context, world).unwrap_or_default();
                let thickness = thickness.get_float(context, world).unwrap_or_default();
                let alpha = alpha.get_float(context, world).unwrap_or_default();
                let color = color.get_color(context, world).unwrap_or(Color::Rgba {
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
            } => {
                let thickness = thickness.get_float(context, world).unwrap_or(1.0) * 0.05;
                let curvature = curvature.get_float(context, world).unwrap_or(1.0);
                let aa = aa.get_float(context, world).unwrap_or(1.0);
                let color = color.get_color(context, world).unwrap_or_default();
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
                                self.fill_default();
                            }
                        }
                    });
                match self {
                    RepresentationMaterial::None => {}
                    RepresentationMaterial::Shape {
                        shape,
                        fill,
                        size,
                        thickness,
                        alpha,
                        color,
                    } => {
                        ui.horizontal(|ui| {
                            ComboBox::from_label("Shape")
                                .selected_text(shape.to_string())
                                .show_ui(ui, |ui| {
                                    for option in Shape::iter() {
                                        let text = option.to_string();
                                        ui.selectable_value(shape, option, text);
                                    }
                                });
                            ComboBox::from_label("Fill")
                                .selected_text(fill.to_string())
                                .show_ui(ui, |ui| {
                                    for option in Fill::iter() {
                                        let text = option.to_string();
                                        ui.selectable_value(fill, option, text);
                                    }
                                });
                        });

                        ui.label("size:");
                        show_tree(size, context, ui, world);
                        ui.label("thickness:");
                        show_tree(thickness, context, ui, world);
                        ui.label("alpha:");
                        show_tree(alpha, context, ui, world);
                        ui.label("color:");
                        show_tree(color, context, ui, world);
                    }
                    RepresentationMaterial::Text {
                        size,
                        text,
                        color,
                        alpha,
                        font_size,
                    } => {
                        ui.label("size:");
                        show_tree(size, context, ui, world);
                        ui.label("text:");
                        show_tree(text, context, ui, world);
                        ui.label("color:");
                        show_tree(color, context, ui, world);
                        ui.label("alpha:");
                        show_tree(alpha, context, ui, world);
                        ui.label("font size:");
                        ui.add(Slider::new(font_size, 16.0..=48.0));
                    }
                    RepresentationMaterial::Curve {
                        thickness,
                        dilations: _,
                        curvature,
                        aa,
                        color,
                    } => {
                        ui.label("thickness:");
                        show_tree(thickness, context, ui, world);
                        ui.label("curvature:");
                        show_tree(curvature, context, ui, world);
                        ui.label("aa:");
                        show_tree(aa, context, ui, world);
                        ui.label("color:");
                        show_tree(color, context, ui, world);
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
            self.material_entities.push(entity);
        }
        self.unpack_children(world);
        *self.material_entities.first().unwrap()
    }
    fn unpack_children(&mut self, world: &mut World) {
        let parent = *self.material_entities.first().unwrap();
        for child in self.children.iter_mut() {
            child.unpack_materials(world.spawn_empty().set_parent(parent).id(), world);
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
            .id_source(id)
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
                                    new_key.show_editor(ui);
                                    show_tree(value, context, ui, world);
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
            if ui.button("COPY").clicked() {
                save_to_clipboard(&to_string_pretty(self, PrettyConfig::new()).unwrap(), world);
                ui.close_menu();
            }
            if ui.button("PASTE").clicked() {
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
