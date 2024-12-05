pub use super::*;
use bevy::asset::{Assets, Handle};
use bevy::math::{vec2, vec4, Vec4, Vec4Swizzles};
use bevy::prelude::{Commands, Mesh, TransformBundle, Visibility, VisibilityBundle, World};
use bevy::render::mesh::PrimitiveTopology;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy::text::{Text, Text2dBundle};
use bevy::{
    asset::Asset,
    color::LinearRgba,
    reflect::TypePath,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{AsBindGroup, PolygonMode, RenderPipelineDescriptor},
    },
    sprite::Material2d,
};

mod curve;
mod shape;

use bevy::color::Alpha;
pub use curve::*;
pub use shape::*;

pub const BEVY_MISSING_COLOR: LinearRgba = LinearRgba::new(1.0, 0.0, 1.0, 1.0);

#[derive(
    Serialize, Deserialize, Debug, Clone, Display, Default, EnumIter, PartialEq, AsRefStr, Hash,
)]
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
        #[serde(default = "text")]
        text: Expression,
        #[serde(default = "color_e")]
        color: Expression,
        #[serde(default = "f32_one_e")]
        alpha: Expression,
        #[serde(default = "font_size")]
        font_size: i32,
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

fn font_size() -> i32 {
    32
}
fn i32_one_e() -> Expression {
    Expression::Value(VarValue::i32(1))
}
fn f32_one_e() -> Expression {
    Expression::Value(VarValue::f32(1.0))
}
fn f32_zero_e() -> Expression {
    Expression::Value(VarValue::f32(0.0))
}
fn f32_arr_e() -> Vec<Box<Expression>> {
    [
        Box::new(Expression::Value(VarValue::f32(0.0))),
        Box::new(Expression::Value(VarValue::f32(1.0))),
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
    Expression::Var(VarName::color)
}
fn color_arr_e() -> Vec<Box<Expression>> {
    [
        Box::new(Expression::Var(VarName::color)),
        Box::new(Expression::Var(VarName::color)),
    ]
    .into()
}
fn text() -> Expression {
    Expression::S("Sample Text".into())
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter, Display, AsRefStr, Hash)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, EnumIter, Display, AsRefStr, Hash)]
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
        parts: Vec<Box<Expression>>,
        #[serde(default = "color_arr_e")]
        colors: Vec<Box<Expression>>,
    },
    GradientRadial {
        #[serde(default = "vec2_zero_e")]
        center: Expression,
        #[serde(default = "f32_one_e")]
        radius: Expression,
        #[serde(default = "f32_arr_e")]
        parts: Vec<Box<Expression>>,
        #[serde(default = "color_arr_e")]
        colors: Vec<Box<Expression>>,
    },
}

#[derive(
    Debug, Clone, Serialize, Deserialize, PartialEq, Default, EnumIter, Display, AsRefStr, Hash,
)]
pub enum RepShapeType {
    #[default]
    Opaque,
    Line {
        #[serde(default = "f32_one_e")]
        thickness: Expression,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, Hash)]
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
            radius: Expression::Value(VarValue::f32(1.0)),
        }
    }
}

impl Default for RepFill {
    fn default() -> Self {
        Self::Solid {
            color: Expression::Var(VarName::color),
        }
    }
}

impl RepShape {
    pub fn shader_shape(&self) -> ShaderShape {
        match self {
            RepShape::Circle { .. } => ShaderShape::Circle,
            RepShape::Rectangle { .. } => ShaderShape::Rectangle,
        }
    }
}

impl RepFill {
    pub fn shader_fill(&self) -> ShaderShapeFill {
        match self {
            RepFill::Solid { .. } => ShaderShapeFill::Solid,
            RepFill::GradientLinear { .. } => ShaderShapeFill::GradientLinear,
            RepFill::GradientRadial { .. } => ShaderShapeFill::GradientRadial,
        }
    }
}

impl RepShapeType {
    pub fn shader_shape_type(&self) -> ShaderShapeType {
        match self {
            RepShapeType::Opaque => ShaderShapeType::Opaque,
            RepShapeType::Line { .. } => ShaderShapeType::Line,
        }
    }
}

impl RepresentationMaterial {
    pub fn unpack(&self, entity: Entity, commands: &mut Commands) {
        let s = self.clone();
        commands.add(move |world: &mut World| match s {
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
                            font_size: font_size as f32,
                            color: BEVY_MISSING_COLOR.into(),
                            ..default()
                        },
                    ),
                    ..default()
                });
            }
            RepresentationMaterial::Curve { .. } => {
                let mut materials = world.resource_mut::<Assets<CurveMaterial>>();
                let material = CurveMaterial {
                    color: BEVY_MISSING_COLOR,
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
        });
    }
    pub fn update(&self, entity: Entity, world: &mut World) {
        world.flush();
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
                    let padding = 0.0;
                    material.data[1].z = padding;
                    match shape {
                        RepShape::Circle { radius } => {
                            let radius = 1.0;
                            let t = &mut material.data[10];
                            if radius != t.x {
                                refresh_mesh = true;
                            }
                            *t = vec4(radius, radius, 0.0, 0.0);
                        }
                        RepShape::Rectangle { size } => {
                            let size = vec2(1.0, 1.0);
                            let t = &mut material.data[10];
                            if t.xy() != size {
                                refresh_mesh = true;
                            }
                            *t = Vec4::from((size, 0.0, 0.0));
                        }
                    }
                    match fill {
                        RepFill::Solid { color } => {
                            material.colors[0] = color
                                .get_value(entity, world)
                                .unwrap()
                                .get_color()
                                .unwrap()
                                .to_linear()
                        }
                        RepFill::GradientLinear {
                            point1,
                            point2,
                            parts: _,
                            colors: _,
                        } => {}
                        RepFill::GradientRadial {
                            center,
                            radius,
                            parts: _,
                            colors: _,
                        } => {}
                    }
                    material.data[10].z = 1.0;

                    if refresh_mesh {
                        let mesh = world.entity(entity).get::<Mesh2dHandle>().unwrap().clone();
                        if let Some(mesh) = world
                            .get_resource_mut::<Assets<Mesh>>()
                            .unwrap()
                            .get_mut(&mesh.0)
                        {
                            let size = (material.data[10].xy() + vec2(padding, padding))
                                .clamp_length_max(100.0);
                            if size.x > 0.0 && size.y >= 0.0 {
                                *mesh = shape.shader_shape().mesh(size);
                            }
                        }
                    }
                    let _ = world
                        .get_resource_mut::<Assets<ShapeMaterial>>()
                        .unwrap()
                        .insert(handle.id(), material);
                }
            }
            RepresentationMaterial::Text {
                size,
                text,
                color,
                alpha,
                font_size,
            } => {}
            RepresentationMaterial::Curve {
                thickness,
                curvature,
                color,
                dilations,
                aa,
                alpha,
            } => {}
        }
    }
}
