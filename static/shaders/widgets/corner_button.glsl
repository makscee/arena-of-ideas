#include <common.glsl>

varying vec2 uv;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
uniform vec2 u_pos;

void main() {
    uv = a_pos * (1 + u_padding) * u_scale * u_scale;
    vec2 world_pos = u_pos + u_size * uv;
    vec3 pos = u_projection_matrix * u_view_matrix * vec3(world_pos, 1.0);
    gl_Position = vec4(pos.xy, 0.0, pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_texture;
uniform vec4 u_icon_color;
uniform vec2 u_corner;
void main() {
    vec4 color = vec4(0);
    vec2 square_uv = uv - (u_corner * 2 - vec2(1));
    color = alphaBlend(color, mix(vec4(0), u_color, float(abs(square_uv.x) + abs(square_uv.y) < 4 * u_scale)));
    vec2 icon_uv = uv * .6 * (1.2 / u_scale / u_scale) + .5;
    vec4 icon_color = u_icon_color;
    icon_color.a = texture2D(u_texture, icon_uv).x;
    color = alphaBlend(color, icon_color);
    gl_FragColor = color;
}
#endif