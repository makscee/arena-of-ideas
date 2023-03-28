#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 uv;
out float t;
attribute vec2 a_pos;

uniform float u_end_cut = 0;
uniform float u_thickness = 0.5;
uniform vec2 u_from;
uniform vec2 u_to;

void main() {
    vec2 pos = vec2(a_pos.x * .5 + 0.5, a_pos.y);
    uv = pos;
    t = 1 - u_t;
    float height = pos.y * u_thickness;
    float bezier_t = pos.x;

    bezier_t = u_end_cut * .5 + bezier_t * (1. - u_end_cut);
    vec4 bezier = bezierParentPartner(bezier_t, u_from, u_to, vec2(0, 1), 0.5);
    vec2 b_pos = bezier.xy;
    vec2 b_normal = bezier.zw;
    b_pos += b_normal * height * t * t * (bezier_t * .7);

    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(b_pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;
in float t;

void main() {
    vec2 uv = uv;
    float centerDist = abs(uv.y);
    if(centerDist > t)
        discard;
    vec4 col = u_color;
    gl_FragColor = col;
}
#endif