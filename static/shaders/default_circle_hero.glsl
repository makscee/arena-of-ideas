#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

void main() {
    float action_t = smoothstep(ACTION_ANIMATION_TIME, 0, u_time - u_action_time);
    action_t *= action_t;
    v_quad_pos = a_pos * (1.0 + u_padding);
    float size = u_unit_radius + action_t * .2;
    vec2 pos = v_quad_pos * size + u_unit_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;

const float THICKNESS = 0.05;

void main() {
    vec4 col = vec4(1);
    vec2 uv = v_quad_pos;
    float dist = length(uv);
    if(dist > 1.0) {
        gl_FragColor = vec4(0);
        return;
    }
    if(dist > 1.0 - THICKNESS) {
        col = vec4(vec3(0), 1);
    }
    gl_FragColor = col;
}
#endif