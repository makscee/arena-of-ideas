#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    v_quad_pos = a_pos * (1.0 + u_padding * UNITS_SCALE);
    float size = u_unit_radius * UNITS_SCALE;
    vec2 pos = v_quad_pos * size + u_unit_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_previous_texture;
uniform vec4 u_status_tint = vec4(1);
in vec2 v_quad_pos;


void main() {
    commonInit();
    vec2 uv = v_quad_pos;
    vec4 previous_color = texture(u_previous_texture, gl_FragCoord.xy / vec2(textureSize(u_previous_texture, 0)));
    float dist = length(uv);
    if (dist > u_unit_radius) discard;
    vec4 injureTint = vec4(parent_enemy_faction_color, max(.0,1 - u_time + u_injure_time));
    vec4 healTint = vec4(heal_color, max(.0,1 - u_time + u_heal_time));
    vec4 col = alphaBlend(previous_color, injureTint);
    col = alphaBlend(col, healTint);
    gl_FragColor = col;
}
#endif