#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    v_quad_pos = a_pos * (1.0 + u_padding);
    float size = u_unit_radius;
    vec2 pos = v_quad_pos * size + u_unit_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;
#include <particles_uniforms.glsl>

#define p_startPosition_redef
vec2 p_startPosition(int i)
{
    return vec2(rand(i) - 0.5, rand(i+1) - 0.5);
}

#define p_velocityOverT_redef
vec2 p_velocityOverT(int i, float t)
{
    return rotateCW(randCircle(i), t * pi) + p_gravity * t;
}

#include <particles_functions.glsl>

void main() {
    vec2 uv = v_quad_pos;
    float t = 1. - u_spawn;

    vec4 col = vec4(0);

    for (int i = 0; i < p_count; i++)
        col = alphaBlend(col, p_renderParticle(i, uv, t));

    gl_FragColor = col;
}
#endif