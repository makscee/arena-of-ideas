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

const float THICKNESS = 0.01;
const float SPREAD = 0.04;

uniform int u_index;

void main() {
    vec2 uv = uv / 1.2;
    float len = length(uv) - 1. - u_index * .1;
    if(abs(len) > THICKNESS + SPREAD)
        discard;
    vec4 color = vec4(u_color.rgb, smoothstep(THICKNESS + SPREAD, THICKNESS, abs(len)));
    gl_FragColor = color;
}
#endif
