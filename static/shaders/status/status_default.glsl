#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    v_quad_pos = a_pos * (1.0 + padding);
    float size = (u_unit_radius - 0.3) * u_spawn;
    vec2 pos = v_quad_pos * size + u_unit_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_previous_texture;
in vec2 v_quad_pos;
void main() {
    vec2 uv = v_quad_pos;
    vec4 previous_color = texture(u_previous_texture, gl_FragCoord.xy / vec2(textureSize(u_previous_texture, 0)));
    float dist = distance(uv, vec2(0));
    // gl_FragColor = alphaBlend(previous_color, statusTint * float(dist < u_unit_radius));
    gl_FragColor = alphaBlend(previous_color, renderStatusRing(uv, u_status_color.rgb));
}
#endif