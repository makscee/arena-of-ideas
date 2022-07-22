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
    vec2 pos = v_quad_pos * size;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 4.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;

void main() {
    gl_FragColor = vec4(0);
}
#endif