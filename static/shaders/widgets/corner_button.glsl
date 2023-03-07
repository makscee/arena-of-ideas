#include <common.glsl>

varying vec2 uv;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform vec2 u_pos;
uniform vec2 u_size;
uniform vec2 u_corner;

void main() {
    uv = a_pos;
    vec2 world_pos = u_pos + u_size * a_pos;
    vec3 pos = u_projection_matrix * u_view_matrix * vec3(world_pos, 1.0);
    gl_Position = vec4(pos.xy, 0.0, pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
void main() {
    vec4 color = u_color;
    color = alphaBlend(color, texture2D(u_texture, uv));
    gl_FragColor = color * vec4(1, 0, 0, 1);
}
#endif