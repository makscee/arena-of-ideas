use bevy::render::render_resource::{PolygonMode, RenderPipelineDescriptor};

use super::*;

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "e50d51b6-c3bd-44fd-a3d4-149afb164c3d"]
pub struct CurveMaterial {
    #[uniform(0)]
    pub color: Color,
}

impl Material2d for CurveMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/curve.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayout,
        _: bevy::sprite::Material2dKey<Self>,
    ) -> __private::Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        descriptor.primitive.polygon_mode = PolygonMode::Fill;
        Ok(())
    }
}

impl Default for CurveMaterial {
    fn default() -> Self {
        Self { color: Color::PINK }
    }
}
