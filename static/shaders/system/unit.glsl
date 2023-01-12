#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

uniform float u_size;

void main() {
    v_quad_pos = a_pos * 1.0;
    vec2 pos = v_quad_pos * 1.0 * u_size;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;

void main() {
    vec4 color = u_color;
    color = vec4(0.6);
    color = vec4(player_faction_color, 1);
    gl_FragColor = color;
}
#endif
