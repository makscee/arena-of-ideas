#include <common.glsl>
#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;
uniform float u_angle_offset;
uniform float u_damage_taken = 0;
uniform float u_animate_on_damage = 0;

void main() {
    init_fields();
    offset = rotate_cw(vec2(0, -1), PI * (.23 - card * .07) * u_angle_offset) * 1.2 * (1 + card * 1.5);
    box = vec2(1 + card * .7);
    uv = get_uv(a_pos);
    size *= (1 + u_damage_taken * u_animate_on_damage);
    gl_Position = get_gl_position(uv);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;

uniform sampler2D u_text;
uniform vec2 u_text_size;

uniform vec4 u_text_color_default;
uniform vec4 u_text_color_decreased;
uniform vec4 u_text_color_increased;
uniform vec4 u_circle_color;

uniform float u_text_scale = 1;
uniform int u_value_modified;

const float BORDER = 0.08;
const float TEXT_INSIDE = 0.5;
const float TEXT_BORDER = 0.37;
const float AA = 0.05;

void main() {
    vec2 uv = uv / (1 - u_card * .1);
    float dist = length(uv);
    vec4 color = vec4(0);
    color = alpha_blend(color, vec4(u_outline_color.rgb, smoothstep(BORDER + AA, BORDER - AA, abs(1 - dist))));
    color = alpha_blend(color, vec4(u_circle_color.rgb, smoothstep(1 - BORDER + AA, 1 - BORDER, dist)));
    vec4 text_color = u_text_color_default;
    if(u_value_modified < 0) {
        text_color = u_text_color_decreased;
    } else if(u_value_modified > 0) {
        text_color = u_text_color_increased;
    }

    float text_scale = u_text_scale;
    float sdf = get_text_sdf(uv / text_scale * vec2(u_text_size.y / u_text_size.x, 1), u_text);
    text_color = get_text_color(sdf, text_color, u_outline_color, TEXT_BORDER, TEXT_INSIDE);
    gl_FragColor = alpha_blend(color, text_color);
}
#endif
