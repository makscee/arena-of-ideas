#include <common.glsl>

uniform vec2 u_size = vec2(0.15, 0.15);
#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform float u_offset = -0.9;

void main() {
    v_quad_pos = a_pos;
    vec2 pos = v_quad_pos * u_size;
    pos.x = mix(pos.x, 0., float(v_quad_pos.y > 0));
    pos += u_unit_position + vec2(0,u_offset);
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;

void main() {
    commonInit();
    vec4 col = vec4(getColor().rgb, 1);
    gl_FragColor = col;
}
#endif
