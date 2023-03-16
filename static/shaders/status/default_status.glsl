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

const float THICKNESS = 0.01;
const float SPREAD = 0.04;

void main() {
    vec2 uv = get_card_uv(uv, u_card);
    uv /= u_scale;
    float len = length(uv) - 1.;
    if(abs(len) > THICKNESS + SPREAD)
        discard;
    vec4 color = vec4(u_color.rgb, smoothstep(THICKNESS + SPREAD, THICKNESS, abs(len)));
    gl_FragColor = color;
}
#endif
