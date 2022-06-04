#include <common.glsl>


#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    const float padding = 1.;
    v_quad_pos = a_pos * (1.0 + padding);
    float size = u_unit_radius;
    vec2 pos = v_quad_pos * size + u_unit_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;

void main() {
    vec2 uv = v_quad_pos;
    float t = u_time;
    vec4 col = vec4(0,0,0,0);
    p_discardCheck(uv, t);
    for (int i = 0; i < p_count; i++)
        col = alphaBlend(col, p_renderParticle(i, uv, t));

    gl_FragColor = col;
}
#endif