use bevy::render::{mesh::MeshVertexBufferLayout, render_resource::RenderPipelineDescriptor};
use convert_case::Casing;
use strum_macros::Display;

use super::*;

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "ec09cb82-5a6b-43cd-ab8a-56d0979f7cc4"]
#[bind_group_data(ShapeMaterialKey)]
pub struct ShapeMaterial {
    #[uniform(0)]
    pub size: Vec2,
    #[uniform(0)]
    pub thickness: f32,
    #[uniform(0)]
    pub alpha: f32,
    #[uniform(0)]
    pub point1: Vec2,
    #[uniform(0)]
    pub point2: Vec2,
    #[uniform(0)]
    pub colors: [Color; 10],
    pub shape: Shape,
    pub fill: Fill,
    pub fill_color: FillColor,
}

#[derive(
    Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Display, Serialize, Deserialize, EnumIter,
)]
pub enum Shape {
    #[default]
    Rectangle,
    Circle,
}

#[derive(
    Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Display, Serialize, Deserialize, EnumIter,
)]
pub enum Fill {
    #[default]
    Line,
    Opaque,
}

#[derive(
    Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Display, Serialize, Deserialize, EnumIter,
)]
pub enum FillColor {
    #[default]
    Solid,
    GradientLinear2,
}

impl Shape {
    pub fn def(&self) -> String {
        self.to_string().to_uppercase()
    }
    pub fn mesh(&self, size: Vec2) -> Mesh {
        match self {
            Shape::Rectangle => Mesh::from(bevy::render::mesh::shape::Quad::new(size)),
            Shape::Circle => Mesh::from(bevy::render::mesh::shape::Circle::new(size.x)),
        }
    }
}

impl Fill {
    pub fn def(&self) -> String {
        self.to_string().to_uppercase()
    }
}

impl FillColor {
    pub fn def(&self) -> String {
        self.to_string()
            .from_case(convert_case::Case::Pascal)
            .to_case(convert_case::Case::UpperSnake)
    }
}

impl Material2d for ShapeMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/sdf_shape.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayout,
        key: bevy::sprite::Material2dKey<Self>,
    ) -> __private::Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let fragment = descriptor.fragment.as_mut().unwrap();
        fragment
            .shader_defs
            .push(key.bind_group_data.shape.def().into());
        fragment
            .shader_defs
            .push(key.bind_group_data.fill.def().into());
        fragment
            .shader_defs
            .push(key.bind_group_data.fill_color.def().into());
        Ok(())
    }
}

#[derive(Eq, PartialEq, Hash, Clone, Copy)]
pub struct ShapeMaterialKey {
    shape: Shape,
    fill: Fill,
    fill_color: FillColor,
}

impl From<&ShapeMaterial> for ShapeMaterialKey {
    fn from(material: &ShapeMaterial) -> Self {
        Self {
            shape: material.shape,
            fill: material.fill,
            fill_color: material.fill_color,
        }
    }
}

impl Default for ShapeMaterial {
    fn default() -> Self {
        Self {
            colors: default(),
            thickness: 1.0,
            alpha: 1.0,
            size: default(),
            shape: default(),
            fill: default(),
            fill_color: default(),
            point1: default(),
            point2: default(),
        }
    }
}
