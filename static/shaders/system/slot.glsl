#include <common.glsl>
uniform float u_hovered;

#ifdef VERTEX_SHADER
out vec2 uv;
out vec2 size;
attribute vec2 a_pos;

void main() {
    size = mix(u_size, u_size * vec2(1, 1.5), u_hovered);
    uv = get_uv(a_pos);
    gl_Position = get_gl_position(a_pos);
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
