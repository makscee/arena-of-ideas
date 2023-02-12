#include <common.glsl>
uniform vec2 u_position = vec2(0);
uniform float u_card;

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

uniform float u_padding = 1;

void main() {
    uv = a_pos * (1.0 + u_padding + u_card);
    vec2 pos = uv * 1.0 + u_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;

uniform int u_hp_current;
uniform int u_hp_max;
uniform float u_hp_last_dmg;

const float THICKNESS = 0.04;
const float SPREAD = 0.2;
const float GLOW = 0.3;
const float DMG_T_DURATION = 3;

const vec2 CARD_SIZE = vec2(1.0, 1.5);
const vec2 CARD_OFFSET = vec2(0, 0.6);
const float CARD_BORDER = 0.07;
const float CARD_AA = 0.1;
const float TEXT_INSIDE = 0.5;
const float TEXT_BORDER = 0.25;

vec4 draw_card(vec4 unit_color, vec2 unit_uv) {
    vec2 uv = uv - CARD_OFFSET;
    float card_sdf = rectangle_sdf(uv, CARD_SIZE, 0);
    commonInit(u_position + uv);
    vec4 color = vec4(0);
    vec4 border_color = vec4(base_color, abs(card_sdf) < CARD_BORDER || (abs(uv.y) < CARD_BORDER * .5 && card_sdf < 0));

    vec2 text_uv = (uv * 1.2 + vec2(0, .5)) * vec2(1, u_texture_size.x / u_texture_size.y);
    // return vec4(abs(text_uv.x) < 1 && abs(text_uv.y) < 1);
    float text_sdf = get_text_sdf(text_uv);
    vec3 text_base_color = vec3(1);
    vec4 text_color = get_text_color(text_sdf, vec4(text_base_color, 1), vec4(text_base_color, .4), TEXT_BORDER, TEXT_INSIDE);
    color = alphaBlend(color, unit_color);
    color = alphaBlend(color, border_color);
    color = alphaBlend(color, text_color);
    return mix(unit_color, color, u_card);
}

void main() {
    vec2 uv = get_card_uv(uv, u_card);
    float len = length(uv) - 1.;
    float hp_part = float(u_hp_current) / float(u_hp_max);
    float dmg_t = (DMG_T_DURATION - u_game_time + u_hp_last_dmg) / DMG_T_DURATION;
    dmg_t = dmg_t * dmg_t * dmg_t;
    commonInit(u_position + uv);
    float alpha = max(smoothstep(THICKNESS, THICKNESS * .5, abs(len)), GLOW * smoothstep(THICKNESS + SPREAD, THICKNESS, abs(len)));
    vec4 color = vec4(base_color, alpha);
    if(len > THICKNESS + SPREAD)
        color.a = 0;
    if(dmg_t > 0. && len < 0.) {
        vec2 v = floor(uv * 8 * (0.5 + dmg_t));
        float r = N22(v + vec2(floor(u_global_time * 20) / 20)).x;
        color = alphaBlend(color, vec4(r, r, r, dmg_t));
    }
    gl_FragColor = draw_card(color, uv);
}
#endif
