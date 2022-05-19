#include <common.glsl>


#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    v_quad_pos = a_pos * (1.0 + padding);
    float size = (u_unit_radius - 0.3) * u_spawn;
    vec2 pos = v_quad_pos * size + u_unit_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;

vec4 renderRing(float rad, float h)
{
    float t = rad / pi / 2.;

    vec4 glowColor = vec4(mixColors(1. - t - 1. / alCountF / 2.), glowValue((h - c_thickness) / c_glowRange));
    vec4 insideColor = vec4(colors[int((pi * 2. - rad) / (pi * 2. / alCountF))], 1.);

    vec4 col = float(h > c_thickness && h - c_thickness < c_glowRange) * glowColor
        + float(h < c_thickness) * insideColor;
    return col;
}

vec4 renderTriangleParticles(vec2 uv, float triangleSize, float t, vec3 color)
{
    if (triangleDist(uv, triangleSize * (1 - t * 2)) > 1. || t == 1.) return vec4(0);

    const float spread = 3;
    float radius = clamp(1 - e_invSquare(t), 0., 1.);
    uv += normalize(uv * t * 0.5);
    uv *= spread;
    uv = vec2(0.5) - fract(uv);
    uv *= uv;
    return vec4(color, float(fract(uv.x) + fract(uv.y) < radius * radius));
}

vec4 renderAbilityReady(vec2 uv, vec3 color)
{
    vec2 center = vec2(0,u_unit_radius + 0.65);
    float radius = 0.35;
    float dist = distance(uv,center);
    return u_ability_ready * float(dist < radius) * vec4(color,1. - dist / radius * dist / radius);
}

void main() {
    commonInit();
    vec2 uv = v_quad_pos;
    float dist = distance(uv,vec2(0.0,0.0));
    vec4 col = vec4(0.,0.,0.,0.);
    float u_action_time = u_action_time - float(u_action_time == 0) * 1000;


    //start

    const float triangleGrowMax = 0.7;
    float fadeAnimation = min(0.5, u_cooldown);
    float triangleSize = -1.7;

    // u_action = e_invSquare(u_action);
    float rotation = -vecAngle(u_face_dir);
    vec2 preRotUv = uv;
    uv = rotateCW(uv, rotation);
    float distToCircle = abs(dist - u_unit_radius);
    col = float(distToCircle < c_thickness + c_glowRange) * renderRing(vecAngle(rotateCW(uv, -pi / u_alliance_count)), distToCircle);

    float animationProgress = clamp((u_time - u_action_time) / fadeAnimation, 0., 1.); // 0 -> 1
    float cooldownProgress = clamp((u_time - u_action_time) / u_cooldown, 0., 1.); // 0 -> 1
    triangleSize *= mix(1. - u_action * u_action, 1., triangleGrowMax);
    float tDist = triangleDist(uv, triangleSize);
    col = alphaBlend(col, vec4(colors[0], float(tDist < 1. && cooldownProgress == 1.)));
    col = alphaBlend(col, renderTriangleParticles(uv - vec2(0,e_invSquare(animationProgress)) * 1.5, triangleSize * triangleGrowMax, animationProgress, colors[0]));
    col = alphaBlend(col, renderAbilityReady(preRotUv, colors[0]));

    //end

    gl_FragColor = col;
}
#endif