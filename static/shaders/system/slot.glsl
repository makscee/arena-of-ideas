#include <common.glsl>
uniform vec2 u_position = vec2(0);
uniform vec2 u_size = vec2(1.1, 0.15);

#ifdef VERTEX_SHADER
out vec2 uv;
out vec2 size;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform vec2 u_offset = vec2(0.0, -1.4);

void main() {
    uv = a_pos;
    size = mix(u_size, u_size * vec2(1, 10), u_hovered);
    vec2 pos = uv * 1.0 * size + u_position + u_offset + vec2(0, size.y);
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
in vec2 size;
uniform float u_filled = 0.0;
uniform float u_border = 0.06;

void main() {
    vec4 color = u_color;
    vec2 border = abs(uv) * size;
    color.a = mix(0.0, 1.1, mix(1, u_filled, float(border.y < size.y - u_border && border.x < size.x - u_border)));
    gl_FragColor = color;
}
#endif
