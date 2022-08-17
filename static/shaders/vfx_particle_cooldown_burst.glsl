#include <common.glsl>
#include <particles_uniforms.glsl>

#ifdef VERTEX_SHADER
attribute vec2 a_pos;

uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

out vec2 v_quad_pos;
flat out int p_index;
flat out float p_t;

void main() {
    float t = 1 - u_spawn;
    p_t = t;
    p_index = gl_InstanceID;

    v_quad_pos = a_pos;
    float size = u_unit_radius;

    vec2 edgePos = randCircle(p_index);
    vec2 startPos = u_parent_radius * edgePos;
    vec2 velocity = edgePos * 2 * (1 - rand(p_index + 3)) + vec2(-edgePos.y, -edgePos.x) * rand(p_index + 2) * 3;
    vec2 pos = u_parent_position + startPos + velocity * t + a_pos * size;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;
flat in float p_t;
flat in int p_index;

void main() {
    if (length(v_quad_pos) > 1) discard;
    commonInit();
    gl_FragColor = vec4(u_color.rgb, 1 - p_t);
}
#endif