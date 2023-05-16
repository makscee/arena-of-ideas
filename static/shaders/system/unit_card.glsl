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

const vec2 CARD_SIZE = vec2(1.0, 1.5);
const float CARD_BORDER = 0.07;
const float CARD_AA = 0.1;
const float TEXT_INSIDE = 0.5;
const float TEXT_BORDER = 0.25;

in vec2 uv;

uniform sampler2D u_description;
uniform vec2 u_description_size;
uniform vec4 u_faction_color;

void main() {
    init_fields();
    vec2 uv = uv / mix(3, 1, u_card);
    float card_sdf = rectangle_sdf(uv * CARD_SIZE.y / CARD_SIZE.x, CARD_SIZE, 0);
    if(card_sdf > CARD_BORDER) {
        discard;
    }
    float border_dist = min(abs(card_sdf) - CARD_BORDER, ((abs(uv.y) - CARD_BORDER) * float(card_sdf < 0)));
    vec4 mixed_color = mix(u_house_color, u_faction_color, smoothstep(0.7, .1, -border_dist / CARD_BORDER));
    vec4 border_color = vec4(mixed_color.rgb, border_dist < 0);

    vec2 text_uv = uv * 2 + vec2(0, 1.0);
    text_uv *= vec2(1, u_description_size.x / u_description_size.y);
    float text_sdf = get_text_sdf(text_uv, u_description);
    vec3 text_base_color = vec3(1);
    vec3 outline_color = vec3(0);
    vec4 text_bg = vec4(field_color, uv.y < 0);
    vec4 text_color = get_text_color(text_sdf, vec4(text_base_color, 1), vec4(outline_color, .7), TEXT_BORDER, TEXT_INSIDE);
    vec4 color = vec4(field_color, 0);
    color = alpha_blend(color, text_bg);
    color = alpha_blend(color, border_color);
    color = alpha_blend(color, text_color);
    color.a = min(color.a, u_card);
    gl_FragColor = color;
}
#endif
