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
    vec2 uv = warp(uv, u_global_time);
    float sdf = fbm_sdf(circle_sdf(uv, 1.0), uv);
    vec4 color = sdf_gradient(sdf);
    gl_FragColor = color;
}
#endif
