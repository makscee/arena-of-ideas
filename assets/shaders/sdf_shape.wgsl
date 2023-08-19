#import bevy_sprite::mesh2d_view_bindings globals
#import bevy_sprite::mesh2d_view_bindings view
#import bevy_sprite::mesh2d_vertex_output MeshVertexOutput

struct SdfShapeMaterial {
    color: vec4<f32>,
    size: vec2<f32>,
};

@group(1) @binding(0)
var<uniform> material: SdfShapeMaterial;

#ifdef RECTANGLE
fn sdf(uv: vec2<f32>, size: vec2<f32>) -> f32 {
    var d = abs(uv) - size;
    return length(max(d,vec2(0.0))) + min(max(d.x,d.y),0.0);
}
#endif
#ifdef CIRCLE
fn sdf(uv: vec2<f32>, size: vec2<f32>) -> f32 {
    var d = length(uv) - size.x;
    return d;
}
#endif

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let uv = (in.uv - vec2(0.5)) * material.size * 2.0;
    let v = f32(sdf(uv, material.size) > -0.1);
    return vec4<f32>(material.color.rgb, v + 0.5);
}