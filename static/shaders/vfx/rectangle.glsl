#include <common.glsl>
uniform float u_sdf_cut = 0;
uniform float u_rounding = 0.3;

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;

void main() {
    init_fields();
    uv = get_uv(a_pos);
    gl_Position = get_gl_position(uv);
    uv *= box;
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;

uniform float u_alpha = 1;
uniform float u_sdf_border = 0.5;
uniform float u_aa = 0.005;
uniform float u_gradient = 0.5;
uniform vec4 u_color_end;

void main() {
    float sdf = rectangle_rounded_sdf(uv, u_box, vec4(u_rounding));
    float alpha = mix(0, u_alpha, (u_sdf_cut - sdf) / u_aa);
    float border = aliase(u_sdf_border, u_sdf_cut, u_aa, sdf);
    vec4 color = u_color;
    float t = u_game_time * (1. + u_rand * .25);
    vec2 uv = rotate_cw(uv, fbm(uv + u_rand * 100.0) * 10. + u_game_time * .05);
    float fbm_v = fbm(uv + t * 0.2);
    float gradient_value = length(uv) * u_gradient * (1.0 + fbm_v * (1. + sin(t * .5) * .2));
    color = mix(u_color, u_color_end, gradient_value);
    color = mix(color, u_outline_color, border);
    color.a = alpha;
    gl_FragColor = color;
}
#endif
