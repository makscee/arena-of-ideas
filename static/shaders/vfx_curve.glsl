#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

uniform vec2 u_p0;
uniform vec2 u_p1;
uniform vec2 u_p2;
uniform vec2 u_p3;

uniform float u_thickness;

vec2 toBezier(float t, vec2 P0, vec2 P1, vec2 P2, vec2 P3)
{
    float t2 = t * t;
    float one_minus_t = 1.0 - t;
    float one_minus_t2 = one_minus_t * one_minus_t;
    return (P0 * one_minus_t2 * one_minus_t + P1 * 3.0 * t * one_minus_t2 + P2 * 3.0 * t2 * one_minus_t + P3 * t2 * t);
}

vec2 toBezierNormal(float t, vec2 P0, vec2 P1, vec2 P2, vec2 P3)
{
    float t2 = t * t;
    return rotateCW(normalize(
        P0 * (-3 * t2 + 6 * t - 3) +
        P1 * (9 * t2 - 12 * t + 3) +
        P2 * (-9 * t + 6 * t) +
        P3 * (3 * t2)), pi * .5);
}

void main() {
    vec2 p0 = u_p0 + vec2(0,cos(u_time * .5));
    vec2 p1 = u_p1 + vec2(0,sin(u_time));
    vec2 p2 = u_p2 + vec2(0,cos(u_time));
    vec2 p3 = u_p3 + vec2(0,sin(u_time * .3));

    vec2 pos = a_pos;
    v_quad_pos = a_pos;
    pos.y *= u_thickness;
    vec2 b_pos = toBezier(pos.x, p0, p1, p2, p3);
    b_pos += toBezierNormal(pos.x, p0, p1, p2, p3) * pos.y;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(b_pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;

void main() {
    vec2 uv = v_quad_pos;
    float t = 1. - u_spawn;

    vec4 col = vec4(uv,1,1);
    gl_FragColor = col;
}
#endif