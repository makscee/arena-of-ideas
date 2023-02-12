#include <common.glsl>
uniform vec2 u_position = vec2(0);

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

uniform float u_padding = 1;
uniform float u_scale = 1;

void main() {
    uv = a_pos * (1.0 + u_padding);
    vec2 pos = uv * 1.0 * u_scale + u_position;
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

void main() {
    float len = length(uv) - 1.;
    if(len > THICKNESS + SPREAD)
        discard;
    float hp_part = float(u_hp_current) / float(u_hp_max);
    float dmg_t = (DMG_T_DURATION - u_game_time + u_hp_last_dmg) / DMG_T_DURATION;
    dmg_t = dmg_t * dmg_t * dmg_t;
    commonInit(u_position + uv);
    float alpha = max(smoothstep(THICKNESS, THICKNESS * .5, abs(len)), GLOW * smoothstep(THICKNESS + SPREAD, THICKNESS, abs(len)));
    vec4 color = vec4(base_color, alpha);
    if(dmg_t > 0. && len < 0.) {
        vec2 v = floor(uv * 8 * (0.5 + dmg_t));
        float r = N22(v + vec2(floor(u_global_time * 20) / 20)).x;
        color = alphaBlend(color, vec4(r, r, r, dmg_t));
    }
    gl_FragColor = color;
}
#endif
