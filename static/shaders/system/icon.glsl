#include <common.glsl>
uniform vec2 u_position = vec2(0);
uniform vec2 u_size;

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

void main() {
    uv = a_pos;
    vec2 pos = uv * u_size + u_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
uniform sampler2D u_texture;
uniform vec4 u_icon_color;

void main() {
    vec4 color = vec4(0);
    vec2 icon_uv = uv * .5 + .5;
    vec4 icon_color = u_icon_color;
    icon_color.a = texture2D(u_texture, icon_uv).x;
    color = alphaBlend(color, icon_color);
    gl_FragColor = color;
    // gl_FragColor = vec4(1, 0, 1, 1);
}
#endif
