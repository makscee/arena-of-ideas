#import bevy_sprite::{
    mesh2d_view_bindings::globals,
    mesh2d_view_bindings::view,
    mesh2d_vertex_output::VertexOutput,
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let speed = 2.0;
    let pos = in.world_position.xy * 0.5;
    let gray = abs(i32(floor(pos.x) + floor(pos.y))) % 2;
    return vec4<f32>(vec3(f32(gray) * 0.01), 1.0);
}