#include <common.glsl>
uniform vec2 u_text_size;

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;

uniform int u_index = 0;
uniform vec2 u_index_offset = vec2(0);
uniform vec2 u_card_offset = vec2(0);
uniform float u_align;
uniform float u_card_scale = 0;
uniform float u_max_width = 100;

void main() {
    init_fields();
    vec2 rel = vec2(u_text_size.x / u_text_size.y, 1);
    rel = mix(rel, vec2(u_max_width, u_max_width / rel.x), float(rel.x > u_max_width));
    offset += u_index * u_index_offset;
    offset += vec2(-rel.x * .5 * u_align, 0) + u_card_offset * card;
    scale += u_card_scale * card;
    size = rel;
    uv = get_uv(a_pos);
    gl_Position = get_gl_position(uv);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;

uniform vec4 u_text_color;
uniform vec4 u_outline_color;

uniform sampler2D u_text;

uniform float u_text_inside = 0.5;
uniform float u_text_border = 0.3;
const float AA = 0.03;

void main() {
    init_fields();
    vec4 color = vec4(0);
    float sdf = get_text_sdf(uv, u_text);
    vec4 text_color = get_text_color(sdf, u_text_color, u_outline_color, u_text_border, u_text_inside);
    color = alphaBlend(color, text_color);
    gl_FragColor = color;
}
#endif
