#include <common.glsl>
#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;

void main() {
    init_fields();
    uv = get_uv(a_pos);
    gl_Position = get_gl_position(uv);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
uniform float u_thickness = 1;
uniform float u_border = 0;
uniform float u_circle_fbm = 0;
uniform float u_circle_fbm_speed = 1.0;
uniform float u_aa = 0.03;

void main() {
    float sdf = circle_sdf(uv, u_size) + (fbm(uv + vec2(u_game_time * u_circle_fbm_speed)) - 0.5) * u_circle_fbm;
    // float alpha = aliase(1 - u_thickness, 1, u_aa, sdf);
    // vec4 color = vec4(u_color.rgb, alpha);
    vec4 color = sdf_gradient(sdf);
    gl_FragColor = color;
}
#endif
