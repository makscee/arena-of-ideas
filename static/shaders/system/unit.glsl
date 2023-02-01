#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

uniform float u_padding = 1;
uniform vec2 u_position = vec2(0);
uniform float u_scale = 1;

void main() {
    v_quad_pos = a_pos * (1.0 + u_padding);
    vec2 pos = v_quad_pos * 1.0 * u_scale + u_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;

uniform int u_hp_current;
uniform int u_hp_max;

const float THICKNESS = 0.04;
const float SPREAD = 0.1;
const float GLOW = 0.2;

void main() {
    float len = abs(1. - length(v_quad_pos));
    float hp_part = float(u_hp_current) / float(u_hp_max);
    if(len > THICKNESS + SPREAD)
        discard;
    if(len < THICKNESS * .5 && (v_quad_pos.y + 1.) * .5 > hp_part) {
        discard;
    }
    commonInit();
    float alpha = max(smoothstep(THICKNESS, THICKNESS * .5, len), GLOW * smoothstep(THICKNESS + SPREAD, THICKNESS, len));
    vec4 color = vec4(faction_color, alpha);
    gl_FragColor = color;
}
#endif
