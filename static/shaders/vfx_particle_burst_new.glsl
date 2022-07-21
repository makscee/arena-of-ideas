#include <common.glsl>
#include <particles_uniforms.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
flat out int p_index;
void main() {
    v_quad_pos = a_pos * (1.0 + u_padding);
    float size = u_unit_radius;
    p_index = gl_InstanceID;
    float t = 0.5;
    vec2 startPos = randVec(p_index) - vec2(0.5);
    vec2 vel = randCircle(p_index + 1) * cos(t*2*pi*rand(p_index));
    vec2 velocity = rotateCW(vel, t * t * pi * .5 * (1. - length(vel))) * 6 * sin(t * .33 * pi);
    vec2 pos = v_quad_pos * size * (1) + startPos + velocity * t;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 4.0);

    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
#include <particles_functions.glsl>
in vec2 v_quad_pos;
flat in int p_index;

void main() {
    vec2 uv = v_quad_pos;
    vec4 col = vec4(randCircle(p_index),rand(p_index),0.9);
    col *= float(p_distToShape(uv) < 0.);
    gl_FragColor = col;
}
#endif