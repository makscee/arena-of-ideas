#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

void main() {
    v_quad_pos = a_pos;
    vec2 pos = v_quad_pos * 1.0;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;
uniform sampler2D u_text_texture;

void main() {
    vec2 uv = v_quad_pos;
    uv += vec2(2, 1);
    uv *= vec2(.2, 1);
    gl_FragColor = texture2D(u_text_texture, uv) * 2.;
    // gl_FragColor = vec4(uv, 0, 1);
}
#endif
