#include <common.glsl>
#include <particles_uniforms.glsl>

#ifdef VERTEX_SHADER
attribute vec2 a_pos;

uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform int u_trail_count = 10;

out vec2 v_quad_pos;
flat out int p_index;
flat out float p_t;

void main() {
    float t = 1 - u_spawn;
    p_index = gl_InstanceID;
    int trail_index = gl_InstanceID % u_trail_count;
    // p_trail_part = float(trail_index) / float(u_trail_count);
    p_index -= trail_index;
    float trail_shift = 0.01 * trail_index;
    t += trail_shift;
    p_t = t;

    v_quad_pos = a_pos;
    float size = u_unit_radius * (1 - t);

    float rpi2 = rand(p_index) * pi * 2 + t * 3;
    vec2 startPos = u_parent_position + vec2(cos(rpi2), sin(rpi2)) * u_parent_radius * (1 - t) * (1 + rand(p_index + 7) * 0.3);
    vec2 pos = startPos + v_quad_pos * size;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;
flat in float p_t;

void main() {
    if (p_t > 1 || length(v_quad_pos) > 1) discard;
    gl_FragColor = u_color;
}
#endif