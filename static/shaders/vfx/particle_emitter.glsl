#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;

uniform int u_trail_count = 1;
uniform float u_lifetime = 1;
uniform float u_trail_shift = 0.01;
uniform float u_velicity_mul = 25;
uniform float u_velocity_over_t = 1;
uniform float u_vel_fbm = 0;
uniform float u_p_size;
uniform float u_p_size_over_t;

flat out int p_index;
flat out float p_t;

void main() {
    init_fields();
    int trail_index = gl_InstanceID % u_trail_count;
    float trail_shift = u_trail_shift * trail_index;
    p_index = gl_InstanceID - trail_index;
    float time = u_game_time + u_lifetime * rand(p_index);
    p_t = time / u_lifetime - floor(time / u_lifetime) - trail_shift;
    p_t += mix(0.0, 1.0, float(p_t < 0.));
    uv = get_uv(a_pos);
    vec2 vel = rotate_cw((rand_vec(p_index + 1) - vec2(0.5)), p_t * PI * u_velocity_over_t);
    vel = vec2(sign(vel.x) * vel.x * vel.x, sign(vel.y) * vel.y * vel.y);
    vel *= u_velicity_mul;
    vel *= 1. + fbm(vec2(u_game_time * 3) + rand_vec(p_index)) * u_vel_fbm;
    box *= u_p_size + u_p_size_over_t * p_t;
    gl_Position = get_gl_position(uv + vel * p_t);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
flat in int p_index;
flat in float p_t;

uniform vec4 u_start_color;
uniform vec4 u_end_color;
uniform float u_alpha = 1;
uniform float u_size_over_t = 0;
uniform float u_p_scale = 1;

void main() {
    float dist = length(uv);
    if(dist > u_p_scale || p_t < 0 || p_t > 1)
        discard;
    gl_FragColor = vec4(mix(u_start_color, u_end_color, p_t).rgb, u_alpha);
}
#endif
