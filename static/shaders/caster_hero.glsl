#include <common.glsl>

varying vec2 v_quad_pos;

#ifdef VERTEX_SHADER
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    v_quad_pos = a_pos * (1.0 + padding);
    float size = (u_unit_radius - 0.3) * u_spawn * (1.0 - 0.25 * u_action);
    vec2 pos = v_quad_pos * size + u_unit_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER

float getTriangleAlpha(vec2 uv, float ang, float size, float thickness)
{
    const float tan3 = tan(pi/3.0);
    const float innerRad = sqrt(3.0) / 6.0;
    vec2 tuv = vec2(
        uv.x * cos(ang) + uv.y * sin(ang),
        -uv.x * sin(ang) + uv.y * cos(ang));
    
    return float(
                   abs(tuv.y + innerRad - (tuv.x + size) * tan3 - tan3 / 2.0) < thickness
                || abs(tuv.y + innerRad + (tuv.x - size) * tan3 - tan3 / 2.0) < thickness
                || abs(tuv.y + innerRad + size) < thickness * .5
                )
        * float(tuv.y + innerRad < (tuv.x + size) * tan3 + tan3 / 2.0 && tuv.y + innerRad < -(tuv.x - size) * tan3 + tan3 / 2.0 && tuv.y + innerRad > -size);
}

void main() {
    vec2 uv = v_quad_pos;

    vec3 colors[3];
    colors[0] = u_alliance_color_1.rgb;
    colors[1] = u_alliance_color_2.rgb;
    colors[2] = u_alliance_color_3.rgb;

    
    float anim = animationFunc(u_action) / 4.;
    
    float dist = distance(uv,vec2(0.0,0.0));
    
    vec4 col = vec4(0.,0.,0.,0.);
    const float timeShift = 0.13;
    float angShift = 0.05 + cos(u_time * .33);
    const float sizeShift = 0.15;
    const float lineThickness = 0.1;
    float ang = 0.;

    for (int i = 0; i < u_alliance_count * 3; i++)
    {
        vec3 curCol = colors[int(mod((i + u_alliance_count * 100), u_alliance_count))];
        float curAlpha = getTriangleAlpha(uv, ang + float(i + 1) * angShift, 0.1 + float(i) * sizeShift * sin(u_time + timeShift * float(i)), lineThickness);
        col = alphaBlend(col, vec4(curCol, curAlpha));
    }

    col.a = 0.4 - (dist - 1.) / glow;
    col = alphaBlend(col, vec4(colors[0],getTriangleAlpha(uv,ang,0.,100.)));
    col = alphaBlend(col, vec4(colors[1],float(u_alliance_count > 1 && dist < 1.) * getTriangleAlpha(uv,ang,0.2,.2)));
    col = alphaBlend(col, vec4(colors[2],float(u_alliance_count > 2 && dist < 1.) * getTriangleAlpha(uv,ang,0.4,.2)));

    if (dist > 1.0 - thickness && dist < 1.0 + thickness) {
        col = alphaBlend(col, vec4(colors[0],1));
    }
    // else if (dist > 1.0 && dist < 1.0 + glow)
    // {
    //     float v = (dist - 1.0) / glow;
    //     col = alphaBlend(col, vec4(colors[0], mix(glowStart, glowEnd, v)));
    // }

    gl_FragColor = col;
}
#endif