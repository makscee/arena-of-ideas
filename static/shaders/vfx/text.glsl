#include <common.glsl>
uniform vec2 u_text_size;

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

uniform float u_padding = 1;
uniform vec2 u_position = vec2(0);
uniform float u_scale = 1;
uniform float u_time;

void main() {
    uv = a_pos * (1.0 + u_padding);
    float u_time = u_time * u_time;
    vec2 rel = vec2(u_text_size.x / u_text_size.y, 1);
    vec2 vel = normalize(u_position) + vec2(0, 1) + vec2(0, -1) * u_time;
    vec2 pos = uv * rel * 1.0 * u_scale * (1 - u_time) + u_position + vel * u_time * 4.;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;

uniform vec4 u_text_color;
uniform vec4 u_outline_color;

uniform sampler2D u_text;

const float TEXT_INSIDE = 0.5;
const float TEXT_BORDER = 0.35;
const float AA = 0.03;

void main() {
    vec4 color = vec4(0);
    vec2 text_uv = uv / vec2(min(1, u_text_size.x / u_text_size.y), 1);
    float sdf = get_text_sdf(text_uv, u_text);
    vec4 text_color = get_text_color(sdf, u_text_color, u_outline_color, TEXT_BORDER, TEXT_INSIDE);
    gl_FragColor = alphaBlend(color, text_color);
}
#endif
