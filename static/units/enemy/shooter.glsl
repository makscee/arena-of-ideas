#include <common.glsl>

varying vec2 v_quad_pos;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    const float padding = 0.15;
    v_quad_pos = a_pos * (1.0 + padding);
    float size = u_unit_radius * u_spawn * (1.0 - 0.25 * u_action);
    vec2 pos = v_quad_pos * size + u_unit_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
void main() {
    const float thickness = 0.05;
    const float glow = 0.15;
    const int segments = 6;
    vec3 c1 = vec3(1.0, 0.3, 0.6);
    vec3 c2 = vec3(0.0, 0.5, 0.8);
    float centerShift = (sin(u_time * 2.0) + 1.0) / 8.0;

    float dist = distance(v_quad_pos, vec2(0.0, 0.0));
    float distShifted = smoothstep(centerShift, 1.0, dist);
    float val = dist;
    
    vec3 col = vec3(1.0, 1.0, 1.0);
    if (dist < 1.0 - thickness)
    {
        if (mod(floor(distShifted * float(segments) + 0.5), 2.0) == 1.0)
            col = c1;
        else col = c2;
    }
    else if (dist > 1.0 - thickness && dist < 1.0)
        col = vec3(0.0, 0.0, 0.0);
    else if (dist > 1.0 && dist < 1.0 + glow)
    {
        float v = (dist - 1.0) / glow;
        col = vec3(v, v, v);
    }

    // Output to screen
    gl_FragColor = vec4(col, 1.0);
}
#endif