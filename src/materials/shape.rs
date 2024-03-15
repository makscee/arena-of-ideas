use bevy::render::{mesh::MeshVertexBufferLayout, render_resource::RenderPipelineDescriptor};
use convert_case::Casing;
use strum_macros::Display;

use super::*;

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "ec09cb82-5a6b-43cd-ab8a-56d0979f7cc4"]
#[bind_group_data(ShapeMaterialKey)]
pub struct ShapeMaterial {
    #[uniform(0)]
    pub colors: [Color; 11],
    #[uniform(0)]
    pub data: [Vec4; 11],
    pub shape: ShaderShape,
    pub shape_type: ShaderShapeType,
    pub shape_fill: ShaderShapeFill,
}

#[derive(
    Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Display, Serialize, Deserialize, EnumIter,
)]
pub enum ShaderShape {
    #[default]
    Rectangle,
    Circle,
}

#[derive(
    Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Display, Serialize, Deserialize, EnumIter,
)]
pub enum ShaderShapeType {
    #[default]
    Opaque,
    Line,
}

#[derive(
    Debug, Clone, Copy, Default, Eq, PartialEq, Hash, Display, Serialize, Deserialize, EnumIter,
)]
pub enum ShaderShapeFill {
    #[default]
    Solid,
    GradientLinear,
    GradientRadial,
}

impl ShaderShape {
    pub fn def(&self) -> String {
        self.to_string().to_uppercase()
    }
    pub fn mesh(&self, size: Vec2) -> Mesh {
        match self {
            ShaderShape::Rectangle => Mesh::from(bevy::render::mesh::shape::Quad::new(size)),
            ShaderShape::Circle => Mesh::from(bevy::render::mesh::shape::Circle::new(size.x)),
        }
    }
}

impl ShaderShapeType {
    pub fn def(&self) -> String {
        self.to_string().to_uppercase()
    }
}

impl ShaderShapeFill {
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
    shape: ShaderShape,
    fill: ShaderShapeType,
    fill_color: ShaderShapeFill,
}

impl From<&ShapeMaterial> for ShapeMaterialKey {
    fn from(material: &ShapeMaterial) -> Self {
        Self {
            shape: material.shape,
            fill: material.shape_type,
            fill_color: material.shape_fill,
        }
    }
}

impl Default for ShapeMaterial {
    fn default() -> Self {
        Self {
            shape: default(),
            shape_type: default(),
            shape_fill: default(),
            colors: default(),
            data: default(),
        }
    }
}
