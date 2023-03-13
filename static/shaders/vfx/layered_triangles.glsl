#include <common.glsl>
#ifdef VERTEX_SHADER
out vec2 uv;
flat out int p_index;
attribute vec2 a_pos;

uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform vec2 u_position;
uniform float u_padding = 0;
uniform float u_radius = 1;
uniform float u_size = 1;

void main() {
    p_index = gl_InstanceID;
    uv = a_pos * (1.0 + u_padding);
    vec2 pos = uv * u_radius * u_size;
    pos = rotateCW(pos, 0.0);
    pos = get_card_pos(pos, u_card);
    pos *= u_zoom;
    pos += u_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
flat in int p_index;

uniform vec4 u_color_1 = vec4(0);
uniform float u_alpha = 0.5;
uniform float u_spin_part = 0.01;
uniform float u_move_speed = 1;

void main() {
    float fbm_v = fbm(randVec(p_index) * vec2(u_global_time * u_move_speed));
    float size_phase = sin(u_global_time + p_index * .3) - fbm_v * .3;
    float sdf = triangleSDF(uv, 0.7 + size_phase * .3, p_index * u_spin_part * fbm_v);
    if(sdf > 0) {
        discard;
    }
    vec4 color = u_color_1;
    color = alphaBlend(color, vec4(u_color.rgb, size_phase));
    color.a = u_alpha;
    gl_FragColor = color;
}
#endif
