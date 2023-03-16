#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;

void main() {
    uv = get_uv(a_pos);
    gl_Position = get_gl_position(a_pos);
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
