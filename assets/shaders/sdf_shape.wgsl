#import bevy_sprite::mesh2d_view_bindings globals
#import bevy_sprite::mesh2d_view_bindings view
#import bevy_sprite::mesh2d_vertex_output MeshVertexOutput

struct ShapeMaterial {
    size: vec2<f32>,
    thickness: f32,
    alpha: f32,
    point1: vec2<f32>,
    point2: vec2<f32>,
    colors: array<vec4<f32>, 10>,
};

@group(1) @binding(0)
var<uniform> material: ShapeMaterial;

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

fn get_line_t(a: vec2<f32>, b: vec2<f32>, p: vec2<f32>) -> f32 {
    var t = dot(p - b, a - b) / dot(a - b, a - b);
    return clamp(t, 0.0, 1.0);
}

const GLOW = 0.1;
const AA = 0.01;
@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let uv = (in.uv - vec2(0.5)) * material.size * 2.0;
    let sdf = sdf(uv, material.size) - AA;
    let thickness = material.thickness * 0.03;
    var v = f32(sdf > -thickness);
#ifdef LINE
    v = min(v, smoothstep(0.0, AA, -sdf)) * v;
    v = max(v, smoothstep(AA, 0.0, -sdf - thickness));
#else ifdef OPAQUE
    v = f32(sdf < 0.0);
#endif
    v = max(v, smoothstep(thickness + GLOW, thickness, -sdf) * 0.1);
#ifdef SOLID
    let color = material.colors[0].rgb;
#else ifdef GRADIENT_LINEAR_2
    let ba = material.point2 - material.point1;
    let a = material.point1 + ba * material.colors[0].a;
    let b = material.point1 + ba * material.colors[1].a;
    let t = get_line_t(a, b, uv);
    let color = mix(material.colors[0].rgb, material.colors[1].rgb, t);
#endif
    return vec4<f32>(color, v * material.alpha);
}   