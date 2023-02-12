#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

uniform float u_padding = 1;
uniform vec2 u_position = vec2(0);
uniform vec2 u_offset = vec2(0);
uniform float u_scale = 1;

void main() {
    uv = a_pos * (1.0 + u_padding);
    vec2 rel = vec2(u_texture_size.x / u_texture_size.y, 1);
    vec2 pos = uv * rel * 1.0 * u_scale + u_position + u_offset;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;

uniform vec4 u_text_color;
uniform vec4 u_outline_color;

const float TEXT_INSIDE = 0.45;
const float TEXT_BORDER = 0.15;

void main() {
    vec4 color = vec4(0);
    vec4 outline_color = u_outline_color;
    float sdf = get_text_sdf(uv);
    vec4 text_color = get_text_color(sdf, u_text_color, outline_color, TEXT_BORDER, TEXT_INSIDE);
    gl_FragColor = alphaBlend(color, text_color);
}
#endif
