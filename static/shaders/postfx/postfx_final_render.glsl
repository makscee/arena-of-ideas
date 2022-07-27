#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
void main() {
    v_quad_pos = a_pos;
    gl_Position = vec4(a_pos.xy, 0.0, 1.);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_frame_texture;
in vec2 v_quad_pos;

void main() {
    vec2 textureSize = textureSize(u_frame_texture, 0);
    vec4 col = texture(u_frame_texture, gl_FragCoord.xy / textureSize);
    gl_FragColor = col;
}
#endif