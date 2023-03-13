#include <common.glsl>
uniform vec2 u_position = vec2(0);
uniform vec2 u_size = vec2(1.2, 1.6);
uniform float u_hovered;

#ifdef VERTEX_SHADER
out vec2 uv;
out vec2 size;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform vec2 u_offset = vec2(0.0, 0.0);

void main() {
    uv = a_pos;
    size = mix(u_size, u_size * vec2(1, 1.5), u_hovered);
    vec2 pos = uv * 1.0 * size + u_position + u_offset;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
in vec2 size;
uniform float u_filled = 0.0;
uniform float u_border = 0.03;
uniform float u_corners = 0.5;

void main() {
    vec4 color = u_color;
    vec2 center_distance = abs(uv) * size;
    float border = u_border * (1.0 + u_filled + u_hovered * 2);
    color.a = mix(0.0, 1.0, float((center_distance.y > size.y - border || center_distance.x > size.x - border) && abs(center_distance.x - size.x) < u_corners && abs(center_distance.y - size.y) < u_corners));
    gl_FragColor = color;
}
#endif
