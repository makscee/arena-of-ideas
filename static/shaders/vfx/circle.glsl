#include <common.glsl>
#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;
uniform float u_circle_radius = 1;

void main() {
    uv = get_uv(a_pos);
    gl_Position = get_gl_position(a_pos, u_radius * u_circle_radius);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
uniform float u_border = 1;
uniform float u_circle_fbm = 0;
uniform float u_circle_fbm_speed = 0;
uniform float u_aa = 0.03;

void main() {
    float len = length(uv) + fbm(uv + vec2(u_game_time * u_circle_fbm_speed)) * u_circle_fbm;
    float alpha = aliase(1 - u_border, 1, u_aa, len);
    vec4 color = vec4(u_color.rgb, alpha);
    gl_FragColor = color;
}
#endif
