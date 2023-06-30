#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;

flat out int p_index;
flat out float p_t;

uniform float u_lifetime = 1;
uniform float u_p_vel_gravity = 0;
uniform float u_p_vel_center = 0;
uniform float u_p_scale = 0.05;
uniform float u_p_scale_start = 0.1;
uniform float u_p_scale_end = 0.9;
uniform vec2 u_p_offset;

void main() {
    init_fields();
    uv = get_uv(a_pos);

    p_index = gl_InstanceID;
    float r1 = rand(p_index);
    float r2 = rand(p_index + 1);
    float r3 = rand(p_index + 2);

    float time = u_game_time + u_lifetime * r1;
    p_t = time / u_lifetime - floor(time / u_lifetime);

    vec2 start_pos = rand_circle(p_index - 1) * box.x * r2;
    vec2 velocity = vec2(0, -1) * u_p_vel_gravity * box.x - start_pos * u_p_vel_center;
    position += start_pos + velocity * p_t + u_p_offset * box.x;
    box *= u_p_scale * smoothhump(u_p_scale_start, u_p_scale_end, p_t);
    gl_Position = get_gl_position(uv);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
flat in int p_index;
flat in float p_t;

void main() {
    vec4 color = sdf_gradient(p_t);
    color.a *= float(length(uv) < 1.);
    gl_FragColor = color;
}
#endif
