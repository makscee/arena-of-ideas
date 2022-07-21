#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform float u_start_scale = 1;
uniform float u_end_scale = 0;
uniform int u_trail_count = 5;

flat out int p_index;
flat out float p_t;

void main() {
    v_quad_pos = a_pos * (1.0 + u_padding);
    p_index = gl_InstanceID;
    int trail_index = gl_InstanceID % u_trail_count;
    p_index -= trail_index;

    float[] r = float[3] (rand(p_index), rand(p_index + 1), rand(p_index + 2));
    float[] r_mid = float[3] (r[0] - 0.5, r[1] - 0.5, r[2] - 0.5);
    float t = 1. - u_spawn * (1 + r[1] * .1);
    float trail_shift = 0.02 * trail_index;
    t -= trail_shift;
    t = 1 - (1 - t) * (1 - t);
    p_t = t;

    float size = u_unit_radius * mix(u_start_scale, u_end_scale, t) * r[2];
    vec2 startPos = rotateCW(vec2(0,u_parent_radius * c_units_scale), pi / 6 * (1 - float(u_parent_position.x < 0) * 2.) + r_mid[0] * 0.2);
    vec2 velocity = rotateCW(startPos, r_mid[1] * 0.5) * r[2] * 6. + vec2(sin(t * r_mid[0] * 9), cos(t * r_mid[1] * 2)) * 1.1;
    velocity *= .6;
    vec2 pos = u_parent_position + v_quad_pos * size + startPos + velocity * t;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);

    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;
flat in int p_index;
flat in float p_t;

void main() {
    if (p_t <= 0.) discard;
    vec2 uv = v_quad_pos;
    vec4 col = vec4(u_color.rgb,0.9);
    col *= float(length(uv) < 0.5);
    gl_FragColor = col;
}
#endif