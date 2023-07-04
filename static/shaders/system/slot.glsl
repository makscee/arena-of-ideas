#include <common.glsl>
uniform float u_hovered;

#ifdef VERTEX_SHADER
out vec2 uv;
out vec2 o_box;
attribute vec2 a_pos;

void main() {
    init_fields();
    box = mix(box, box * vec2(1, 1.5), u_hovered);
    o_box = box;
    uv = get_uv(a_pos);
    gl_Position = get_gl_position(uv);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
in vec2 o_box;
uniform float u_filled = 0.0;
uniform float u_border = 0.04;
uniform float u_corners = 0.7;

void main() {
    vec4 color = u_color;
    vec2 center_distance = abs(uv) * o_box;
    float border = u_border * (1.0 + u_filled * 2 + u_hovered * 2);
    color.a = mix(0.0, 1.0, float((center_distance.y > o_box.y - border || center_distance.x > o_box.x - border) && abs(center_distance.x - o_box.x) < u_corners && abs(center_distance.y - o_box.y) < u_corners));
    gl_FragColor = color;
}
#endif
