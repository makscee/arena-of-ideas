#include <common.glsl>

varying vec2 v_quad_pos;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    const float padding = 1.;
    v_quad_pos = a_pos * (1.0 + padding);
    float size = u_unit_radius * u_spawn * (1.0 - 0.25 * u_action);
    vec2 pos = v_quad_pos * size + u_unit_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER

vec3 getSpikesColor(vec2 uv, vec3 col, float height, float spread, float timeShift)
{
    float a = 0.5;
    float m = 4. + spread;
    float x = uv.x * m + timeShift;
    return col * float(uv.y < -abs(x - round(x)) * a + a / 4. + height + 0.2);
}

void main() {
    const int segments = 5;
    vec2 uv = v_quad_pos;
    vec3 colors[3];
    colors[0] = u_alliance_color_1.rgb;
    colors[1] = u_alliance_color_2.rgb;
    colors[2] = u_alliance_color_3.rgb;
    
    float anim = animationFunc(u_action) / 2.;
    
    float dist = distance(uv,vec2(0.0,0.0));
    
    vec4 col = vec4(0.);
    if (dist < 1.0 - thickness)
    {
        const float heightShift = 0.17;
        const float timeShift = 0.3;
        col = vec4(colors[2],1.);
        for (float i = 0.; i > -10.; i--)
        {
            float curTimeShift = u_time + i * timeShift;
            vec3 sc = getSpikesColor(uv, colors[mod(int(i + 3003.), 3)],
                heightShift * i + heightShift * anim - (float(i < 0.) * heightShift * 1.5) + cos(curTimeShift * 1.13) * .1, 0.5 + anim, sin(curTimeShift) * .5);
            if (sc != vec3(0.))
                col = vec4(sc,1.);
        }
    }
    else if (dist > 1.0 - thickness && dist < 1.0 + thickness) {
        col = vec4(colors[0],1.);
    }
    else if (dist > 1.0 && dist < 1.0 + glow)
    {
        float v = (dist - 1.0) / glow;
        col = vec4(colors[0], mix(glowStart, 0., v));
    } else {
        col.a = 0.0;
    }

    gl_FragColor = col;
}
#endif