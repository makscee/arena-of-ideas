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

const float THICKNESS = 0.04;
const float SPREAD = 0.2;
const float GLOW = 0.4;

in vec2 uv;

uniform float u_damage_taken;
uniform vec4 u_faction_color;
uniform int u_rank;

void main() {
    float len = length(uv) - 1.;
    len += sin(vec_angle(uv) * 20 + u_game_time * 3) * (0.01 + 0.05 * (u_rank));
    float dmg_t = u_damage_taken;
    vec4 color = vec4(0);
    float thickness = THICKNESS;
    float alpha = max(smoothstep(thickness, thickness * .5, abs(len)), GLOW * smoothstep(thickness + SPREAD, thickness, abs(len)));
    color = alpha_blend(color, vec4(u_faction_color.rgb * (1. - u_rank * 0.2), alpha));
    if(len > thickness + SPREAD)
        color.a = 0;
    if(dmg_t > 0. && len < 0.) {
        vec2 v = floor(uv * 8 * (0.5 + dmg_t));
        float r = n22(v + vec2(floor(u_global_time * 20) / 20)).x;
        color = alpha_blend(color, vec4(r, r, r, dmg_t));
    }
    gl_FragColor = color;
}
#endif
