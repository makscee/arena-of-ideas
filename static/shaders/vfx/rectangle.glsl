#include <common.glsl>
uniform float u_sdf_cut = 0;

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;

uniform vec2 u_align = vec2(0);

void main() {
    init_fields();
    padding += u_sdf_cut / min(box.x, box.y);
    uv = get_uv(a_pos);
    position += u_align * box;
    gl_Position = get_gl_position(uv);
    uv *= box;
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;

uniform float u_alpha = 1;
uniform float u_sdf_border = 0.5;
uniform float u_aa = 0.005;

void main() {
    float sdf = rectangle_sdf(uv, u_box, 0);
    float alpha = mix(0, u_alpha, sdf < u_sdf_cut);
    float border = aliase(u_sdf_border, u_sdf_cut, u_aa, sdf);
    vec4 color = mix(u_color, u_outline_color, border);
    color.a = alpha;
    gl_FragColor = color;
}
#endif
