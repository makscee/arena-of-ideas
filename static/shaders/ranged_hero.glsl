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

vec3 getAngleColor(vec2 uv, vec3 col, float height)
{
    return col * float(uv.y < -abs(uv.x * .5) + height);
}

void main() {
    vec2 uv = v_quad_pos;
    vec3 colors[3];
    colors[0] = u_alliance_color_1.rgb;
    colors[1] = u_alliance_color_2.rgb;
    colors[2] = u_alliance_color_3.rgb;

    float anim = animationFunc(u_action) / 4.;
    float dist = distance(uv,vec2(0.0,0.0));
    
    vec4 col = vec4(0.);
    if (dist < 1.0 - thickness)
    {
        col = vec4(colors[2],1.);
        float heightShift = 0.2 + anim / 8.;
        const float timeShift = 0.4;
        for (float i = 10.; i >= -12.; i--)
        {
            float h = .35 + i * heightShift - (float(i <= -1.) * heightShift * 1.5) + sin(u_time + timeShift * i) * heightShift;
            vec3 ac = getAngleColor(uv, colors[mod(int(i + 3000.), 3)], h);
            if (ac != vec3(0.,0.,0.))
                col = vec4(ac,1.);
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