#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;

uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform vec2 u_position;
uniform float u_padding = 1;
uniform float u_radius = 1;
uniform float u_size = 1;

void main() {
    uv = a_pos * (1.0 + u_padding);
    vec2 pos = uv * u_radius * u_size;
    pos = get_card_pos(pos, get_card_value());
    pos *= (1 + u_hovered);
    pos += u_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
uniform float u_border = 1;
uniform float u_circle_fbm = 0;
uniform float u_circle_fbm_speed = 0;
uniform float u_aa = 0.03;

void main() {
    float len = length(uv) + fbm(uv + vec2(u_global_time * u_circle_fbm_speed)) * u_circle_fbm;
    float alpha = aliase(1 - u_border, 1, u_aa, len);
    vec4 color = vec4(u_color.rgb, alpha);
    gl_FragColor = color;
}
#endif
