#include <common.glsl>

varying vec2 v_quad_pos;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    const float padding = 1.;
    v_quad_pos = a_pos * (1.0 + padding);
    float size = u_unit_radius;
    vec2 pos = v_quad_pos * size + u_unit_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER

float getRingAlpha(
    vec2 uv, float r, float thickness, float glow, float glowStart, float glowEnd, float innerMult, float outerMult)
{
    float dist = distance(uv, vec2(0.));
    float circleDist = abs(r - dist);
    float halfThickness = thickness * .5;
    glow *= max(r, 0.5);
    return max(float(circleDist < halfThickness),

        float(circleDist > halfThickness && circleDist < halfThickness + glow)
        * mix(glowStart, glowEnd, (circleDist - halfThickness) / glow)
        * (float(dist > r) * outerMult + float(dist < r) * 1.)
        * (float(dist < r) * innerMult + float(dist > r) * 1.));
}

void main() {
    const float final_radius = 0.5;
    const float inside_alpha = 1.0;
    vec2 uv = v_quad_pos;
    vec3 color = vec3(1);
    vec4 col = vec4(color,0.);
    float t = 1. - u_spawn;

    float radius = t * final_radius;
    float dist = distance(uv, vec2(0.));
    float alpha = 1. - t;
    col = alphaBlend(col, vec4(color, alpha * getRingAlpha(uv, radius, 0.1, .3 + radius, 1., 0.1, 1., 1.)));
    col.a = max(col.a, float(dist < radius) * inside_alpha * alpha);

    gl_FragColor = col;
}
#endif