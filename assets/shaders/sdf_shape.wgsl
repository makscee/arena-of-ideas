#import bevy_sprite::mesh2d_view_bindings globals
#import bevy_sprite::mesh2d_view_bindings view
#import bevy_sprite::mesh2d_vertex_output MeshVertexOutput
#import noisy_bevy fbm_simplex_2d

struct ShapeMaterial {
    colors: array<vec4<f32>, 11>,
    data: array<vec4<f32>, 11>,
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
    let d = length(uv) - size.x;
    return d;
}
#endif

fn get_line_t(a: vec2<f32>, b: vec2<f32>, p: vec2<f32>) -> f32 {
    let t = dot(p - a, b - a) / dot(a - b, a - b);
    return clamp(t, 0.0, 1.0);
}
fn get_circle_t(center: vec2<f32>, radius: f32, p: vec2<f32>) -> f32 {
    let t = length(center - p) / radius;
    return clamp(t, 0.0, 1.0);
}

const GLOW = 0.1;
const AA = 0.01;
@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let size = material.data[10].xy;
    let alpha = material.data[10].z;
    let thickness = material.data[10].w * 0.03;

    var uv = (in.uv - vec2(0.5)) * size * 2.0;
#ifdef FBM
    let octaves = i32(material.data[9].x);
    let lacunarity = material.data[9].y;
    let gain = material.data[9].z;
    let offset = material.data[9].w;
    uv += vec2(
        fbm_simplex_2d(uv, octaves, lacunarity, gain),
        fbm_simplex_2d(uv + vec2(offset), octaves, lacunarity, gain)
    );
#endif
    let sdf = sdf(uv, size) - AA;
    var v = f32(sdf > -thickness);
#ifdef LINE
    v = min(v, smoothstep(0.0, AA, -sdf)) * v;
    v = max(v, smoothstep(AA, 0.0, -sdf - thickness));
#else ifdef OPAQUE
    v = f32(sdf < 0.0);
#endif
    v = max(v, smoothstep(thickness + GLOW, thickness, -sdf) * 0.1);
#ifdef SOLID
    var color = material.colors[0];
#else ifdef GRADIENT_LINEAR
    let a = material.data[0].xy;
    let b = material.data[1].xy;
    var t = get_line_t(a, b, uv);
    t = smoothstep(material.data[0].w, material.data[1].w, t);
    var color = mix(material.colors[0], material.colors[1], t);
#else ifdef GRADIENT_RADIAL
    let center = material.data[0].xy;
    let radius = material.data[0].z;
    var t = get_circle_t(center, radius, uv);
    t = smoothstep(material.data[0].w, material.data[1].w, t);
    var color = mix(material.colors[0], material.colors[1], t);
#endif
    color.a *= v * alpha;
    return color;
}   