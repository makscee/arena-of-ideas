#import bevy_sprite::mesh2d_view_bindings globals
#import bevy_sprite::mesh2d_view_bindings view
#import bevy_sprite::mesh2d_vertex_output MeshVertexOutput

struct CurveMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: CurveMaterial;

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let color = material.color.rgb;

    return vec4<f32>(color, 1.0 - abs(in.uv.y));
}