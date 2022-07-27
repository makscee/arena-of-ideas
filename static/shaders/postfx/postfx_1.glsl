#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    v_quad_pos = a_pos;
    gl_Position = vec4(a_pos.xy, 0.0, 1.);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_previous_texture;
uniform ivec2 u_window_size;
uniform float u_sigma = 0.4;
uniform int u_width = 20;
uniform int u_phase = 0;
in vec2 v_quad_pos;

float calcGauss(float x, float sigma) {
  float coeff = 1.0 / (2.0 * 3.14157 * sigma);
  float expon = -(x*x) / (2.0 * sigma);
  return (coeff*exp(expon));
}
void main() {
    vec2 textureSize = textureSize(u_previous_texture, 0);
    vec4 col = texture(u_previous_texture, gl_FragCoord.xy / textureSize);

    vec4 gaussCol = vec4(col.rgb, 1.0);
    vec2 step = 1.0 / textureSize;
    for (int i = 1; i <= u_width; ++ i) {
        vec2 actStep = mix(vec2(i, 0), vec2(0, i), u_phase);
        float weight = calcGauss(float(i) / float(u_width), u_sigma);
        col = texture(u_previous_texture, (gl_FragCoord.xy + actStep) / textureSize);
        gaussCol += vec4(col.rgb * weight, weight);
        col = texture(u_previous_texture, (gl_FragCoord.xy - actStep) / textureSize);
        gaussCol += vec4(col.rgb * weight, weight);
    }
    gaussCol.rgb /= gaussCol.w;
    gaussCol.rgb *= 1.3;
    gl_FragColor = gaussCol;
}
#endif