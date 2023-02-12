#include <common.glsl>
uniform vec2 u_position = vec2(0);

#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

uniform float u_padding = 1;

void main() {
    uv = a_pos * (1.0 + u_padding);
    vec2 pos = uv * 1.0 + u_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;

uniform float u_card;
uniform float u_scale = 1.3;

const float THICKNESS = 0.01;
const float SPREAD = 0.04;

void main() {
    vec2 uv = get_card_uv(uv, u_card) / u_scale;
    float len = length(uv) - 1.;
    if(abs(len) > THICKNESS + SPREAD)
        discard;
    vec4 color = vec4(u_color.rgb, smoothstep(THICKNESS + SPREAD, THICKNESS, abs(len)));
    gl_FragColor = color;
}
#endif
