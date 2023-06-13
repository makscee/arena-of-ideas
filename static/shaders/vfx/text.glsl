#include <common.glsl>
uniform vec2 u_text_size;

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;

uniform vec2 u_position_over_t = vec2(0);
uniform float u_size_over_t = 0;
uniform vec2 u_text_align;

void main() {
    init_fields();
    position += u_position_over_t * u_t;
    box *= vec2(1. + u_size_over_t * u_t);
    vec2 rel = vec2(u_text_size.x / u_text_size.y, 1) * box.y;
    box = rel * min(1.0, box.x / rel.x);
    position += box * u_text_align;
    uv = get_uv(a_pos);
    gl_Position = get_gl_position(uv);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;

uniform sampler2D u_text;

uniform float u_text_inside = 0.5;
uniform float u_text_border = 0.35;
uniform float u_alpha = 1;
uniform float u_alpha_over_t = 0;
uniform float u_outline_fade = 0;
uniform float u_mid_border = 0.0;
uniform float u_outline_fbm = 0.05;
uniform vec4 u_mid_border_color = vec4(0, 0, 0, 1);

void main() {
    init_fields();
    vec4 color = vec4(0);
    float sdf = get_text_sdf(uv, u_text);
    sdf = mix(sdf, min(u_text_inside - u_mid_border, sdf + fbm(uv + vec2(u_game_time, 0)) * u_outline_fbm), float(sdf < u_text_inside - u_mid_border));
    vec4 text_color = get_text_color(sdf, u_color, u_outline_color, u_text_border, u_text_inside);
    float mid_border = smoothstep(u_mid_border, 0., abs(sdf - u_text_inside));
    text_color = mix(text_color, u_mid_border_color, mid_border);
    float outline_fade = smoothstep(u_text_inside, u_text_border, sdf) * u_outline_fade;
    text_color.a -= outline_fade;
    text_color.a = clamp((text_color.a * u_alpha + u_alpha_over_t * u_t) * float(text_color.a > 0.), 0., 1.);
    color = alpha_blend(color, text_color);
    gl_FragColor = color;
}
#endif
