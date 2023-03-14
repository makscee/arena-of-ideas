#include <common.glsl>
uniform vec2 u_text_size;

#ifdef VERTEX_SHADER
out vec2 uv;
out float t;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

uniform float u_padding = 1;
uniform vec2 u_position = vec2(0);
uniform vec2 u_offset = vec2(0);
uniform vec2 u_position_over_t = vec2(0);
uniform float u_scale = 1;
uniform float u_scale_over_t = 0;
uniform float u_gravity = 0;
uniform vec2 u_direction = vec2(0, 1);
uniform float u_velocity = 0;
uniform float u_time = 0;

void main() {
    uv = a_pos * (1.0 + u_padding);
    t = u_time;
    vec2 rel = vec2(u_text_size.x / u_text_size.y, 1);
    vec2 vel = normalize(u_direction) * u_velocity + vec2(0, u_gravity * t);
    vec2 pos = uv * rel * 1.0 * (u_scale + u_scale_over_t * t) + u_position + u_offset + vel * t + u_position_over_t * t;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
in float t;

uniform vec4 u_text_color;
uniform vec4 u_outline_color;
uniform float u_alpha_over_t = 0;

uniform sampler2D u_text;

const float TEXT_INSIDE = 0.5;
const float TEXT_BORDER = 0.35;
const float AA = 0.03;

void main() {
    vec4 color = vec4(0);
    float sdf = get_text_sdf(uv, u_text);
    vec4 text_color = get_text_color(sdf, u_text_color, u_outline_color, TEXT_BORDER, TEXT_INSIDE);
    color = alphaBlend(color, text_color);
    color.a += u_alpha_over_t * t;
    gl_FragColor = color;
}
#endif
