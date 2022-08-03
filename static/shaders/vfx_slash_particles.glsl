#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

flat out int p_index;
flat out float bezier_t;

void main() {
    p_index = gl_InstanceID;

    v_quad_pos = a_pos;
    float effect_t = 1 - u_spawn;
    bezier_t = rand(p_index) * 1.0 + effect_t * .15;

    vec2 p0 = u_parent_position;
    vec2 p1 = p0 + vec2(1, 0) * u_parent_faction;
    vec2 p3 = p0 + vec2(0.7 * u_parent_faction, 1.5);
    vec2 p2 = p1 + vec2(0.7 * u_parent_faction, 1);
    vec2 b_pos = toBezier(bezier_t, p0, p1, p2, p3);
    vec2 b_normal = toBezierNormal(bezier_t, p0, p1, p2, p3);

    vec2 startPos = b_pos + b_normal * (0.5 - rand(p_index + 2)) * bezier_t;
    vec2 velocity = b_normal * (0.5 - rand(p_index + 1));
    // velocity *= .1;
    float radius = u_unit_radius * bezier_t * smoothstep(0.95, 0.6, effect_t);

    vec2 pos = v_quad_pos * radius + startPos + velocity * effect_t;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;
flat in float p_index;
flat in float bezier_t;

void main() {
    float t = 1 - u_spawn;
    if (t < bezier_t * .1 || length(v_quad_pos) > 1) discard;
    commonInit();
    vec4 col = vec4(parent_faction_color, 1);
    gl_FragColor = col;
}
#endif