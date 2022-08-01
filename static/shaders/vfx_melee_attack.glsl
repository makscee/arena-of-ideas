#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

flat out float bezier_t;

void main() {
    vec2 pos = vec2(a_pos.x * .5 + 0.5, a_pos.y);
    v_quad_pos = pos;
    float height = pos.y * u_thickness;
    bezier_t = pos.x;

    vec2 p0 = u_parent_position;
    vec2 p1 = p0 + vec2(1, 0) * u_parent_faction;
    vec2 p3 = p0 + vec2(0.5 * u_parent_faction, 1.5);
    vec2 p2 = p1 + vec2(0.5 * u_parent_faction, 1);
    vec2 b_pos = toBezier(bezier_t, p0, p1, p2, p3);
    vec2 b_normal = toBezierNormal(bezier_t, p0, p1, p2, p3);

    b_pos += b_normal * height * (bezier_t * 3.3);

    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(b_pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;
flat in float bezier_t;

void main() {
    float t = 1 - u_spawn;
    if (t < bezier_t * .1) discard;
    commonInit();
    vec2 uv = v_quad_pos;
    float v = 1 + bezier_t * .15 - t * t;
    v = max(.7 - t * .7, v);
    vec4 col = vec4(parent_faction_color, v);
    gl_FragColor = col;
}
#endif