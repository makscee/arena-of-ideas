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

const float SIZE = 1.0;

void main() {
    init_fields();
    vec2 uv = get_card_uv(uv);
    float len = length(uv);
    if(length(uv) > SIZE) {
        discard;
    }
    float len_fbm = length(vec2(fbm(uv * u_size + vec2(u_game_time * 2, sin(u_game_time))) * 3));
    vec4 color = vec4(u_color.rgb, len_fbm * (1 - len));
    gl_FragColor = color;
}
#endif
