#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;

void main() {
    init_fields();
    uv = get_uv(a_pos);
    gl_Position = get_gl_position(uv);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;

void main() {
    gl_FragColor = vec4(uv, 1., 1.);
}
#endif
