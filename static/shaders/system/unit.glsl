#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;

void main() {
    init_fields();
    card = 0;
    uv = get_uv(a_pos);
    gl_Position = get_gl_position(uv);
}
#endif

#ifdef FRAGMENT_SHADER

const float THICKNESS = 0.04;
const float SPREAD = 0.2;
const float GLOW = 0.4;
const float DMG_T_DURATION = 3;

const vec2 CARD_SIZE = vec2(1.0, 1.5);
const float CARD_BORDER = 0.07;
const float CARD_AA = 0.1;
const float TEXT_INSIDE = 0.5;
const float TEXT_BORDER = 0.25;

in vec2 uv;

uniform float u_hovered;
uniform float u_damage_taken;
uniform sampler2D u_description;
uniform vec2 u_description_size;
uniform vec4 u_house_color1;

vec4 draw_card(vec4 unit_color, vec2 unit_uv) {
    vec2 uv = uv / mix(3, 1, u_card);
    float card_sdf = rectangle_sdf(uv * CARD_SIZE.y / CARD_SIZE.x, CARD_SIZE, 0);
    if(card_sdf > CARD_BORDER) {
        return vec4(0);
    }
    float border_dist = min(abs(card_sdf) - CARD_BORDER, ((abs(uv.y) - CARD_BORDER) * float(card_sdf < 0)));
    vec3 mixed_color = mix(u_house_color1.rgb, base_color, smoothstep(.3, 1, -border_dist / CARD_BORDER));
    vec4 border_color = vec4(mixed_color, border_dist < 0);

    vec2 text_uv = uv * 2 + vec2(0, 1.0);
    text_uv *= vec2(1, u_description_size.x / u_description_size.y);
    float text_sdf = get_text_sdf(text_uv, u_description);
    vec3 text_base_color = vec3(1);
    vec3 outline_color = vec3(0);
    vec4 text_bg = vec4(field_color, uv.y < 0);
    vec4 text_color = get_text_color(text_sdf, vec4(text_base_color, 1), vec4(outline_color, .7), TEXT_BORDER, TEXT_INSIDE);
    vec4 card_color = vec4(field_color, 0);
    card_color = alphaBlend(card_color, text_bg);
    card_color = alphaBlend(card_color, border_color);
    card_color = alphaBlend(card_color, text_color);
    card_color.a = min(card_color.a, u_card);
    return alphaBlend(unit_color, card_color);
}

void main() {
    init_fields();
    vec2 uv = get_card_uv(uv);
    float len = length(uv) - 1.;
    float dmg_t = u_damage_taken;
    vec4 color = vec4(field_color, 0);
    float alpha = max(smoothstep(THICKNESS, THICKNESS * .5, abs(len)), GLOW * smoothstep(THICKNESS + SPREAD, THICKNESS, abs(len)));
    color = alphaBlend(color, vec4(base_color, alpha));
    if(len > THICKNESS + SPREAD)
        color.a = 0;
    if(dmg_t > 0. && len < 0.) {
        vec2 v = floor(uv * 8 * (0.5 + dmg_t));
        float r = N22(v + vec2(floor(u_global_time * 20) / 20)).x;
        color = alphaBlend(color, vec4(r, r, r, dmg_t));
    }
    color.a *= (1 + u_hovered);
    gl_FragColor = draw_card(color, uv);
}
#endif
