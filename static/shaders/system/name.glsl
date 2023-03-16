#include <common.glsl>
uniform vec2 u_name_size;

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;
uniform float u_height = 0.15;
uniform float u_width = 0.6;

void main() {
    uv = get_uv(a_pos);
    vec2 rel = vec2(u_name_size.x / u_name_size.y, 1) * u_height;
    rel *= mix(1., u_width / rel.x, float(rel.x > u_width));
    vec2 pos = uv * rel * 1.0 * u_scale + u_offset + u_card * vec2(0, -0.05);
    pos *= u_zoom;
    pos += u_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;

uniform vec4 u_text_color;
uniform vec4 u_outline_color;
uniform sampler2D u_name;

uniform float u_text_inside = 0.45;
uniform float u_text_outline = 0.15;

void main() {
    vec4 color = vec4(0);
    vec4 outline_color = u_outline_color;
    float sdf = get_text_sdf(uv, u_name);
    vec4 text_color = get_text_color(sdf, u_text_color, outline_color, u_text_outline, u_text_inside);
    gl_FragColor = alphaBlend(color, text_color);
}
#endif
