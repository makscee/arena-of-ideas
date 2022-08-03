#include <common.glsl>

uniform vec2 u_size = vec2(0.9, 0.15);
#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform float u_offset = -0.6;

void main() {
    v_quad_pos = a_pos * u_size;
    vec2 pos = u_unit_position + vec2(0,u_offset) + v_quad_pos * 0.4;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;
uniform float u_empty = 1;

void main() {
    commonInit();
    vec4 col = vec4(parent_faction_color, 1);
    col.a = max(u_empty, float(abs(v_quad_pos.x) > u_size.x - u_thickness || abs(v_quad_pos.y) > u_size.y - u_thickness));
    gl_FragColor = col;
}
#endif
