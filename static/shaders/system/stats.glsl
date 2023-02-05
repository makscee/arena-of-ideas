#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

uniform float u_padding = 1;
uniform vec2 u_position = vec2(0);
uniform float u_scale = 1;
uniform float u_offset;

void main() {
    v_quad_pos = a_pos * (1.0 + u_padding);
    vec2 pos = v_quad_pos * 1.0 * u_scale + u_position + rotateCW(vec2(0, -1), PI * .25 * u_offset);
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;
uniform sampler2D u_text_texture;
uniform vec2 u_texture_size;
uniform float u_size;
uniform int u_hp_current;
uniform int u_hp_max;

const float BORDER = 0.1;

void main() {
    float dist = length(v_quad_pos);
    if(dist > 1.)
        discard;
    float hp_part = float(u_hp_current) / float(u_hp_max);
    if(dist < 1. - BORDER && dist > hp_part * (1. - BORDER)) {
        discard;
    }
    gl_FragColor = vec4(0.82f, 0.06f, 0.06f, 1.);
}
#endif
