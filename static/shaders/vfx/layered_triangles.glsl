#include <common.glsl>
#ifdef VERTEX_SHADER
out vec2 uv;
flat out int p_index;
attribute vec2 a_pos;

void main() {
    init_fields();
    p_index = gl_InstanceID;
    uv = get_uv(a_pos);
    gl_Position = get_gl_position(uv);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
flat in int p_index;

uniform vec4 u_color_1 = vec4(0);
uniform float u_alpha = 0.5;
uniform float u_spin_part = 0.01;
uniform float u_move_speed = 0;
uniform float u_size_amp = 1;

void main() {
    float fbm_v = fbm(rand_vec(p_index) * vec2(u_global_time * u_move_speed));
    float size_phase = sin(u_global_time + p_index * .3) - fbm_v * .3;
    float sdf = triangle_sdf(uv, 0.7 + size_phase * .3 * u_size_amp, p_index * u_spin_part * fbm_v);
    if(sdf > 0) {
        discard;
    }
    vec4 color = u_color_1;
    color = alpha_blend(color, vec4(u_color.rgb, size_phase));
    color.a = u_alpha;
    gl_FragColor = color;
}
#endif
