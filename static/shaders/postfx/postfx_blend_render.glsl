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
uniform sampler2D u_previous_texture;
in vec2 v_quad_pos;

void main() {
    vec2 textureSize = textureSize(u_frame_texture, 0);
    vec4 frame = texture(u_frame_texture, gl_FragCoord.xy / textureSize);
    vec4 prev = texture(u_previous_texture, gl_FragCoord.xy / textureSize);
    float frameL = luminance(frame);
    float prevL = luminance(prev);
    gl_FragColor = mix(frame, prev, float(prevL > frameL));
    // gl_FragColor = prev;
}
#endif