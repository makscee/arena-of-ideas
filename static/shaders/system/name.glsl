#include <common.glsl>
uniform vec2 u_texture_size;

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

uniform float u_padding = 1;
uniform vec2 u_position = vec2(0);
uniform vec2 u_offset = vec2(0);
uniform float u_scale = 1;

void main() {
    uv = a_pos * (1.0 + u_padding);
    vec2 rel = vec2(u_texture_size.x / u_texture_size.y, 1);
    vec2 pos = uv * rel * 1.0 * u_scale + u_position + u_offset;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
uniform sampler2D u_text_texture;

uniform vec4 u_text_color;
uniform vec4 u_outline_color;

const float TEXT_INSIDE = 0.58;
const float TEXT_BORDER = 0.25;
const float AA = 0.03;

vec4 get_text_color(float sdf, vec4 text_color, vec4 outline_color) {
    return mix(mix(vec4(0), outline_color, smoothstep(TEXT_BORDER - AA, TEXT_BORDER + AA, sdf)), text_color, smoothstep(TEXT_INSIDE - AA, TEXT_INSIDE + AA, sdf));
}

void main() {
    vec4 color = vec4(0);
    vec2 text_uv = uv / vec2(min(1, u_texture_size.x / u_texture_size.y), 1);
    float sdf = texture2D(u_text_texture, text_uv * .5 + .5).x;
    vec4 outline_color = u_outline_color;
    vec4 text_color = get_text_color(sdf, u_text_color, outline_color);
    gl_FragColor = alphaBlend(color, text_color);
}
#endif
