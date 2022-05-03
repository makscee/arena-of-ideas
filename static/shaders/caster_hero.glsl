#include <common.glsl>


#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
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
in vec2 v_quad_pos;

vec3 mixColors(float t)
{
    t += float(t < 0.);
    int colorInd = int(t * alCountF);
    vec3 c1 = colors[colorInd];
    vec3 c2 = colors[mod(colorInd + 1, int(alCountF))];
    return mix(c1, c2, t * alCountF - float(colorInd));
}

vec4 getRingColor(float rad, float h)
{
    float t = rad / pi / 2.;

    vec4 glowColor = vec4(mixColors(1. - t - 1. / alCountF / 2.), glowValue((h - c_thickness) / c_glowRange));
    vec4 insideColor = vec4(colors[int((pi * 2. - rad) / (pi * 2. / alCountF))], 1.);

    vec4 col = float(h > c_thickness && h - c_thickness < c_glowRange) * glowColor
        + float(h < c_thickness) * insideColor;
    return col;
}

void main() {
    commonInit();
    vec2 uv = v_quad_pos;
    
    float anim = animationFunc(u_action) / 4.;
    float dist = distance(uv,vec2(0.0,0.0));
    vec4 col = vec4(0.,0.,0.,0.);

    float rotation = -vecAngle(u_target_dir);
    uv = rotateCW(uv, rotation);
    float distToCircle = abs(dist - u_unit_radius);
    col = float(distToCircle < c_thickness + c_glowRange) * getRingColor(vecAngle(uv), distToCircle);

    float cooldownLeft = clamp(1. - (u_time - u_action_time - u_animation_delay) / u_cooldown, 0., 1.);
    float tDist = triangleDist(uv, -1.7);
    col = alphaBlend(col, vec4(colors[0], float(tDist < 1. && tDist > 0.97 * cooldownLeft)));

    gl_FragColor = col;
}
#endif