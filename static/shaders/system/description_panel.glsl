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
uniform sampler2D u_title;
uniform vec2 u_title_size;

const float BORDER_THICKNESS = 0.1;

void main() {
    init_fields();
    vec2 border_thickness = BORDER_THICKNESS / u_box;
    vec4 color = u_color;
    float border = float(abs(uv.x) > 1 - border_thickness.x || abs(uv.y) > 1 - border_thickness.y);
    border = max(border, float(abs(uv.y - 0.7) < border_thickness.y * .5));
    color.rgb *= (1. - .2 * border);
    border = border * smoothstep(-.9, .5, uv.y);
    float background = smoothstep(-1.1, 0.3, uv.y);

    vec2 text_uv = uv * 2.5 / vec2(box.y / box.x, 1);
    text_uv.x /= u_title_size.x / u_title_size.y;
    vec2 text_position = vec2(0.0, 2.0);
    text_uv -= text_position;
    float text_sdf = get_text_sdf(text_uv * 1.5, u_title);
    vec4 text_color = get_text_color(text_sdf, u_color, vec4(1), 0.43, 0.5);

    float alpha = max(border, background);
    color.a = alpha;
    color = alpha_blend(color, text_color);
    gl_FragColor = color;
}
#endif
