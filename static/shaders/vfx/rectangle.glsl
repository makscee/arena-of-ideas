#include <common.glsl>
#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;

uniform vec2 u_align = vec2(0);

void main() {
    init_fields();
    uv = get_uv(a_pos);
    position += u_align * box;
    gl_Position = get_gl_position(uv);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;

uniform float u_alpha = 1;

void main() {
    gl_FragColor = vec4(u_color.rgb, u_alpha);
}
#endif
