#include <common.glsl>
uniform float u_time;

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

uniform vec2 u_position = vec2(0);
uniform float u_scale = 1;
uniform int u_trail_count;

flat out int p_index;
flat out float p_t;

void main() {
    int trail_index = gl_InstanceID % u_trail_count;
    float trail_shift = 0.002 * trail_index;
    p_index = gl_InstanceID - trail_index;
    p_t = u_time + trail_shift;
    v_quad_pos = a_pos;
    vec2 vel = (randCircle(p_index) + sin(randVec(p_index) * PI * 2 + p_t * 1.5)) * rand(p_index + 1) * 2.5;
    vec2 pos = v_quad_pos * 1.0 * u_scale + u_position + vel * p_t;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;
flat in int p_index;
flat in float p_t;

void main() {
    float dist = length(v_quad_pos);
    if(dist > 1. - p_t)
        discard;
    vec3 color = mix(vec3(0), vec3(1), float(p_index % 2));
    gl_FragColor = vec4(color, 1.);
}
#endif
