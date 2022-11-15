#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    v_quad_pos = a_pos * (1.0 + u_padding);
    vec2 pos = v_quad_pos * u_unit_radius + u_unit_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;
uniform int u_shape;

float getRingAlpha(vec2 uv, float r, float thickness, float spread, float glowValue, float glowSpread, float innerMul, float outerMul) {
    float sdf = 0.;
    if(u_shape == 0) {
        sdf = circleSDF(uv, r);
    } else if(u_shape == 1) {
        sdf = squareSDF(uv, r);
    } else if(u_shape == 2) {
        sdf = triangleSDF(uv, r, 0.);
    }
    float asdf = abs(sdf);
    return float(asdf < thickness) + smoothstep(spread, 0., asdf - thickness) + smoothstep(-glowSpread, 0., sdf + thickness) * glowValue * innerMul * float(sdf < -thickness) + smoothstep(glowSpread, 0., sdf - thickness) * glowValue * outerMul * float(sdf > thickness);
}

void main() {
    commonInit();
    vec2 uv = v_quad_pos;
    float size = 1.0;
    vec3 color = getColor().rgb;
    float t = u_time;

    float clock_1 = sin(t);
    float clock_d2 = sin(t / 2);
    float clock_d4 = sin(t / 4);
    float clock_m2 = sin(t * 2);

    float thickness = clock_1 * .01 + .01 + clock_d4 * .01;
    float spread = 0.01;
    float glowValue = clock_d2 * .1 + 0.3;
    float glowSpread = (1. - clock_1) * .1 + 0.4;
    float innerMul = clock_m2 * .3 + 1.;
    float outerMul = clock_m2 * .3 + 0.3;
    vec4 col = vec4(color, getRingAlpha(uv, size, thickness, spread, glowValue, glowSpread, innerMul, outerMul));

    thickness = 0.;
    spread = 0.;
    glowValue = 0.3;
    glowSpread = (1. - clock_1) * .1 + 0.4;
    innerMul = .5 * smoothstep(0.7, 1., sin(t + 0.9));
    outerMul = 0.;

    size = clock_1 * 2.;
    col = alphaBlend(col, vec4(color, getRingAlpha(uv, size, thickness, spread, glowValue, glowSpread, innerMul, outerMul)));

    gl_FragColor = col;
}
#endif