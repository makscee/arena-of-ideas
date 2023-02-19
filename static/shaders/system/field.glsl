#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

void main() {
    uv = a_pos * 100.0;
    vec2 pos = uv * 1.0;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
uniform vec4 u_color_1;
uniform vec4 u_color_2;

void main() {
    float t = get_field_value(uv);
    vec4 color = mix(u_color_1, u_color_2, t);
    gl_FragColor = color;
}
#endif
