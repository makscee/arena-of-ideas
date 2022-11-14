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
    // float t = .999;
    p_index = gl_InstanceID;
    v_quad_pos = a_pos * vec2(2., 1.);

    vec2 startPos = randCircle(p_index) * rand(p_index + 2) * u_parent_radius + u_parent_position;
    vec2 velocity = vec2(0, .5) * (1.5 - rand(p_index + 3));
    float size = 1.4 - rand(p_index + 1) * .5;
    vec2 pos = startPos + velocity * (0.8 * rand(p_index + 1) + invSquare(t)) + a_pos * u_unit_radius * size;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;
flat in int p_index;
flat in float p_t;

void main() {
    if (u_direction.y < 0 && (v_quad_pos.y > 0. || v_quad_pos.y < -1. + abs(v_quad_pos.x))) discard;
    if (u_direction.y > 0 && (v_quad_pos.y < 0. || v_quad_pos.y > 1. - abs(v_quad_pos.x))) discard;
    commonInit();
    vec3 col = u_color.rgb;
    float alpha = .8 * (1. - p_t);

    gl_FragColor = vec4(col, alpha);
}
#endif