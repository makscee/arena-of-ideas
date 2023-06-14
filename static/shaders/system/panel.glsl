#include <common.glsl>
uniform float u_rounding = 0.05;
uniform sampler2D u_title_text;
uniform vec2 u_title_text_size;

const float EXTRA_HEIGHT = 0.1;

#ifdef VERTEX_SHADER
out vec2 uv;
out vec2 o_box;
attribute vec2 a_pos;

void main() {
    init_fields();
    uv = get_uv(a_pos);
    box *= vec2(1, 1. + (EXTRA_HEIGHT) / box.y);
    o_box = box;
    gl_Position = get_gl_position(uv);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
in vec2 o_box;

uniform vec4 u_border_color;
uniform vec4 u_start_color;
uniform vec4 u_end_color;

const float BORDER_THICKNESS = 0.01;

float title_sdf(vec2 uv) {
    float p = EXTRA_HEIGHT * .25;
    vec2 padding = vec2(-p / o_box.x * 2, p / o_box.y * .5);
    uv += vec2(1, -1) + padding;
    uv *= o_box;
    vec2 box = vec2(u_title_text_size.y / u_title_text_size.x, 1) / EXTRA_HEIGHT * 2;
    uv *= box;
    uv += vec2(-1.0, 1.0);
    return get_text_sdf(uv, u_title_text);
}

void main() {
    // vec2 uv = warp(uv, u_global_time);
    float box_sdf = rectangle_rounded_sdf(uv * o_box, o_box, vec4(u_rounding));
    float inner_box_sdf = rectangle_rounded_sdf(uv * o_box, o_box - vec2(0, EXTRA_HEIGHT), vec4(0));
    // sdf = fbm_sdf(sdf, uv);
    vec4 box_body_color = mix(u_start_color, u_end_color, rotate_cw(uv, -PI * .25 + u_global_time * .5).x * 0.5 + 1.);

    vec4 color = vec4(0);

    vec4 box_color = mix(box_body_color, u_border_color, float(abs(box_sdf) < BORDER_THICKNESS && box_sdf < 0));
    box_color.a = smoothstep(0.0, -0.001, box_sdf);

    bool is_border = inner_box_sdf > -BORDER_THICKNESS && inner_box_sdf < 0;
    vec4 body_border_color = vec4(u_border_color.rgb, float(is_border && box_sdf < 0));
    vec4 body_color = vec4(u_color.rgb, float(inner_box_sdf < 0));

    color = alpha_blend(color, box_color);
    color = alpha_blend(color, body_color);
    color = alpha_blend(color, body_border_color);
    float title_sdf = title_sdf(uv);
    vec4 title_color = vec4(0, 0, 0, smoothstep(0.2, 0.5, title_sdf));
    title_color = alpha_blend(title_color, vec4(1, 1, 1, float(title_sdf > 0.5)));
    color = alpha_blend(color, title_color);
    gl_FragColor = color;
}
#endif
