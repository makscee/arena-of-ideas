use super::*;

#[derive(Asset, AsBindGroup, TypePath, Debug, Clone)]
pub struct CurveMaterial {
    #[uniform(0)]
    pub color: Color,
    #[uniform(0)]
    pub aa: f32,
    #[uniform(0)]
    pub alpha: f32,
}

impl Material2d for CurveMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/curve.wgsl".into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _: &MeshVertexBufferLayout,
        _: bevy::sprite::Material2dKey<Self>,
    ) -> serde::__private::Result<(), bevy::render::render_resource::SpecializedMeshPipelineError>
    {
        descriptor.primitive.polygon_mode = PolygonMode::Fill;
        Ok(())
    }
}

impl Default for CurveMaterial {
    fn default() -> Self {
        Self {
            color: Color::PINK,
            aa: 0.0,
            alpha: 1.0,
        }
    }
}
