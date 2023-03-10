#include <common.glsl>
const float BORDER_THICKNESS = 0.025;

uniform vec2 u_position = vec2(0);
uniform vec2 u_offset = vec2(0, 0);
uniform vec2 u_size = vec2(0.5, 0.5);
uniform float u_height = 0.7;

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform float u_padding = 1;

void main() {
    uv = a_pos * (1.0 + BORDER_THICKNESS + u_padding) * vec2(1, u_height);
    vec2 pos = uv * u_size + u_offset;
    pos *= (1 + u_hovered);
    pos += u_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
const float NAME_HEIGHT = 0.4;

in vec2 uv;
uniform sampler2D u_description;
uniform vec2 u_description_size;
uniform sampler2D u_name;
uniform vec2 u_name_size;

void main() {
    float card = get_card_value();
    vec2 uv = uv / (vec2(2) - card);
    commonInit(u_position + u_offset + uv);
    float card_sdf = rectangle_sdf(uv, vec2(1, u_height), 0);
    vec4 color = vec4(field_color, card_sdf < 0);
    float border_value = max(float(BORDER_THICKNESS - abs(card_sdf) > 0), float(BORDER_THICKNESS - abs(uv.y - 1 + NAME_HEIGHT) > 0 && card_sdf < 0));
    float border_glow = (.7 - card_sdf / .5) * float(card_sdf > 0);
    vec4 border_color = vec4(base_color.rgb, max(border_value, border_glow));
    vec2 name_uv = uv * vec2(u_name_size.y / u_name_size.x, 1) - vec2(0, 1 - NAME_HEIGHT * .5);
    name_uv /= NAME_HEIGHT;
    vec4 name_color = get_text_color(get_text_sdf(name_uv * 1.2, u_name), vec4(vec3(1), 1), u_color, .2, .5);
    vec2 description_uv = uv * vec2(1, u_description_size.x / u_description_size.y) + vec2(0, u_height * .1);
    vec4 description_color = get_text_color(get_text_sdf(description_uv * 1.2, u_description), vec4(vec3(1), 1), vec4(vec3(0), 0.8), .25, .5);
    color = alphaBlend(color, border_color);
    color = alphaBlend(color, name_color);
    color = alphaBlend(color, description_color);
    gl_FragColor = vec4(color.rgb, color.a * card * u_hovered);
}
#endif
