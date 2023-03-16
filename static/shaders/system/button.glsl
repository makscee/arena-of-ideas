#include <common.glsl>
uniform float u_hovered;

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;

void main() {
    uv = get_uv(a_pos);
    // size = mix(u_size, u_size * 1.1, u_hovered);
    gl_Position = get_gl_position(a_pos);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
in vec2 size;

void main() {
    gl_FragColor = vec4(1, 0, 1, 1);
}
#endif
