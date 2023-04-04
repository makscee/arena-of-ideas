#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;

void main() {
    init_fields();
    uv = get_uv(a_pos);
    gl_Position = get_gl_position(a_pos);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
uniform sampler2D u_texture;
uniform vec4 u_icon_color;
uniform float u_hovered;

void main() {
    vec4 color = vec4(0);
    vec2 icon_uv = uv * .5 + .5;
    vec4 icon_color = u_icon_color;
    icon_color.a = texture2D(u_texture, icon_uv).x;
    color = alpha_blend(color, icon_color);
    gl_FragColor = color * (1. + u_hovered);
}
#endif
