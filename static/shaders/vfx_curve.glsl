#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform float u_end_cut = 0;


void main() {
    vec2 pos = vec2(a_pos.x * .5 + 0.5, a_pos.y);
    v_quad_pos = pos;
    float height = pos.y * u_thickness;
    float bezier_t = pos.x;

    bezier_t = u_end_cut * .5 + bezier_t * (1. - u_end_cut);
    vec4 bezier = bezierParentPartner(bezier_t, u_parent_position, u_partner_position);
    vec2 b_pos = bezier.xy;
    vec2 b_normal = bezier.zw;
    b_pos += b_normal * height * u_spawn * u_spawn * (1.0 - bezier_t * .7);

    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(b_pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;

void main() {
    commonInit();
    vec2 uv = v_quad_pos;
    float centerDist = abs(uv.y);
    if (centerDist > u_spawn) discard;
    vec4 col = getColor();
    gl_FragColor = col;
}
#endif