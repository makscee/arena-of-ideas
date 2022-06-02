#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    v_quad_pos = a_pos;
    float size = 0.95;
    vec2 pos = v_quad_pos * size;
    vec3 p_pos = vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_previous_texture;
uniform ivec2 u_window_size;
in vec2 v_quad_pos;
void main() {
    vec2 uv = v_quad_pos;
    uv *= 20;
    uv.x *= float(u_window_size.x) / float(u_window_size.y);
    float t = u_time * 0.4;
    uv.x += sin(t) + cos(uv.y * .5 + t);
    uv.y += sin(t * 1.3) + cos(uv.x * 0.2 + t);
    vec4 col = vec4(0.4);
    col *= float(int(floor(uv.x) + floor(uv.y)) % 2 == 0);
    float dist = distance(uv, vec2(0));
    // gl_FragColor = alphaBlend(previous_color, statusTint * float(dist < u_unit_radius));
    gl_FragColor = col;
}
#endif