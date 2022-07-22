#include <common.glsl>
#include <particles_uniforms.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
flat out int p_index;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform float p_disperse = 0;
uniform float u_end_cut = 0;


void main() {
    v_quad_pos = a_pos + vec2(0.0, 0.);
    p_index = gl_InstanceID;
    float r1 = rand(p_index);
    float r2 = rand(p_index + 1);
    float r3 = rand(p_index + 2);
    float effect_t = 1 - u_spawn;
    float bezier_t = r1 + effect_t * r3 * p_speed;
    bezier_t -= float(bezier_t > 1);
    bezier_t = u_end_cut * .5 + bezier_t * (1. - u_end_cut);

    vec4 bezier = bezierParentPartner(bezier_t, u_parent_position, u_partner_position);
    vec2 b_pos = bezier.xy;
    vec2 b_normal = bezier.zw;

    vec2 startPos = b_pos + b_normal * (r2- 0.5) * r3 * u_thickness * (1 + p_disperse * effect_t);
    float radius = u_unit_radius * cos(effect_t * pi * .5) * sin(bezier_t * pi);

    vec2 pos = v_quad_pos * radius + startPos;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);

    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;
flat in int p_index;

void main() {
    commonInit();
    vec2 uv = v_quad_pos;
    float centerDist = distance(uv, vec2(.0));
    // if (centerDist > 0.5) discard;
    vec4 col = vec4(parent_faction_color, 1);
    col.a = float(centerDist < 1) * .5;
    gl_FragColor = col;
}
#endif