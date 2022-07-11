#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;


void main() {
    vec2 pos = a_pos * .5 + vec2(0.5, 0.);
    v_quad_pos = pos;
    pos.y *= u_thickness;

    pos.x = .1 + pos.x * .8;
    vec4 bezier = bezierParentPartner(pos.x, u_parent_position, u_partner_position);
    vec2 b_pos = bezier.xy;
    vec2 b_normal = bezier.zw;
    b_pos += b_normal * pos.y * u_spawn * u_spawn * (1. - pos.x * .7);

    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(b_pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;

void main() {
    vec2 uv = v_quad_pos;
    float centerDist = abs(uv.y);
    vec4 col = u_color;
    col.a = (1. - centerDist) * u_spawn * u_spawn;
    gl_FragColor = col;
}
#endif