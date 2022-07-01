#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    v_quad_pos = a_pos * (1.0 + u_padding);
    float size = u_unit_radius * u_spawn;
    vec2 pos = v_quad_pos * size + u_unit_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_previous_texture;
in vec2 v_quad_pos;

#include <particles_uniforms.glsl>
#define p_renderParticle_redef
#include <particles_functions.glsl>

void p_discardCheck(vec2 uv, float t)
{
    if (uv.y < -2.) discard;
}

vec4 p_renderParticle(int i, vec2 uv, float t)
{
    p_discardCheck(uv, t);
    t = fract(float(i) / p_count + t);
    vec2 pos = p_positionOverT(i, t);
    return vec4(p_colorOverT(i, t),p_alphaOverT(i,t)) * float(p_distToShape(pos, uv, p_radiusOverT(i,t)) < 0.);
}


void main() {
    vec2 uv = v_quad_pos;
    vec4 previous_color = texture(u_previous_texture, gl_FragCoord.xy / vec2(textureSize(u_previous_texture, 0)));
    // float dist = distance(uv, vec2(0));
    // gl_FragColor = alphaBlend(previous_color, statusTint * float(dist < u_unit_radius));
    float t = u_time;
    vec4 col = vec4(0,0,0,0);
    for (int i = 0; i < p_count; i++)
        col = alphaBlend(col, p_renderParticle(i + u_status_index * 100 + u_status_count * 333, uv, t));
    gl_FragColor = alphaBlend(previous_color, alphaBlend(col, renderStatusRing(uv, u_status_color.rgb)));
}
#endif