#include <common.glsl>

uniform float u_card = .5;
#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform vec2 u_position;
uniform float u_scale = 1;
uniform float u_padding = 1;

void main() {
    uv = a_pos * (1.0 + u_padding + u_card);
    vec2 pos = uv * 1.0 + u_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;

const float SIZE = 1.0;

void main() {
    vec2 uv = get_card_uv(uv, u_card);
    float len = length(uv);
    if(length(uv) > SIZE) {
        discard;
    }
    uv += vec2(fbm(uv + vec2(u_game_time * 2, sin(u_game_time))) * 2);
    float len_fbm = length(uv);
    vec4 color = vec4(u_color.rgb, (len_fbm) * (1 - len));
    gl_FragColor = color;
}
#endif
