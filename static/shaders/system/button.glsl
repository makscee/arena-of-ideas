#include <common.glsl>
uniform vec2 u_position = vec2(0);
uniform vec2 u_size;
uniform float u_hovered;

#ifdef VERTEX_SHADER
out vec2 uv;
out vec2 size;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

void main() {
    uv = a_pos;
    size = mix(u_size, u_size * 1.1, u_hovered);
    vec2 pos = uv * size + u_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
in vec2 size;

void main() {
    gl_FragColor = vec4(1, 0, 1, 1);
}
#endif
