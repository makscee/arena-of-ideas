#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 uv;
out float card;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

uniform float u_padding = 1;
uniform vec2 u_position = vec2(0);
uniform float u_scale = 1;
uniform float u_offset;

void main() {
    uv = a_pos * (1.0 + u_padding);
    card = get_card_value();
    vec2 pos = uv * 1.0 * u_scale + rotateCW(vec2(0, -1), PI * (.23 - card * .03) * u_offset) * 1.2 * (1 + card * .05);
    pos *= (1 + u_hovered);
    pos += u_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
in float card;

uniform sampler2D u_text;
uniform vec2 u_text_size;

uniform vec4 u_text_color_default;
uniform vec4 u_text_color_decreased;
uniform vec4 u_text_color_increased;
uniform vec4 u_outline_color;
uniform vec4 u_circle_color;

uniform float u_text_scale = 1;
uniform float u_damage_taken = 0;
uniform int u_value_modified;

const float BORDER = 0.08;
const float TEXT_INSIDE = 0.5;
const float TEXT_BORDER = 0.37;
const float AA = 0.05;
const float CHANGE_T_DURATION = 1.0;

void main() {
    vec2 uv = uv / (1 - card * .1);
    float dist = length(uv);
    vec4 color = vec4(0);
    color = alphaBlend(color, vec4(u_outline_color.rgb, smoothstep(BORDER + AA, BORDER - AA, abs(1 - dist))));
    color = alphaBlend(color, vec4(u_circle_color.rgb, smoothstep(1 - BORDER + AA, 1 - BORDER, dist)));
    vec4 text_color = u_text_color_default;
    if(u_value_modified < 0) {
        text_color = u_text_color_decreased;
    } else if(u_value_modified > 0) {
        text_color = u_text_color_increased;
    }

    float text_scale = u_text_scale * (1 + u_damage_taken);
    float sdf = get_text_sdf(uv / text_scale * vec2(u_text_size.y / u_text_size.x, 1), u_text);
    text_color = get_text_color(sdf, text_color, u_outline_color, TEXT_BORDER, TEXT_INSIDE);
    gl_FragColor = alphaBlend(color, text_color);
}
#endif
