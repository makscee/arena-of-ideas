//#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform vec2 u_position = vec2(0);
uniform float u_scale = 1;

void main() {
    v_quad_pos = a_pos;
    vec2 pos = v_quad_pos * u_scale + u_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;
uniform vec4 u_color;

void main() {
    // vec4 color = mix(u_color, u_color_1, float(v_quad_pos.x < .0));
    if(length(v_quad_pos) > 1.0) {
        discard;
    }
    gl_FragColor = u_color;
}
#endif
