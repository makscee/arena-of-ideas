#include <common.glsl>


#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    v_quad_pos = a_pos * (1.0 + u_padding);
    float size = u_unit_radius * .4;
    vec2 pos = v_quad_pos * size + u_unit_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;

float getRingAlpha(
    vec2 uv, float r, float thickness, float glow, float glowStartV, float innerMult, float outerMult)
{
    float dist = distance(uv, vec2(0.));
    float circleDist = abs(r - dist);
    float halfThickness = thickness * .5;
    glow *= max(r, 0.5);
    return max(float(circleDist < halfThickness),

        float(circleDist > halfThickness && circleDist < halfThickness + glow)
        * mix(glowStartV, 0., (circleDist - halfThickness) / glow)
        * (float(dist > r) * outerMult + float(dist < r) * 1.)
        * (float(dist < r) * innerMult + float(dist > r) * 1.));
}

float getAngleAlpha(vec2 uv, vec3 col, float height)
{
    return clamp(mix(.5, 0., abs(float(uv.y + abs(uv.x * .5) - height) * 5.)) * 1.5, 0., 1.);
}

void main() {
    float glow = 0.35 + sin(u_time) * .1;
    vec2 uv = v_quad_pos;
    float rotation = -vecAngle(u_face_dir);
    uv = rotateCW(uv, rotation);
    vec3 colors[2];
    colors[0] = vec3(0.698, 0.133, 0.133);
    colors[1] = vec3(1, 0.980, 0.941);
    float innerTime = u_time - floor(u_time / pi * 2.) * pi * 2.;

    float outerR = 1. - sin(innerTime) * .05, innerR = 0.8 + sin(innerTime) * .5;

    const float innerFade = 1.2;
    float innerAlpha = float(cos(innerTime) > 0.)
        + float(cos(innerTime + innerFade));
    innerAlpha = clamp(innerAlpha, 0., 1.);

    float innerAlpha2 = -1., innerR2 = -1.;
    float anim = animationFunc(u_action) / 4.;
    anim = 0.;
    innerAlpha2 += float(anim != 0.) * (2. - abs(anim) * .3);
    innerR2 += float(anim != 0.) * (abs(anim) * 1.5 + 1.);

    float distCenter = distance(uv,vec2(0.0,0.0));
    float distOuter = outerR - distCenter;
    float distInner = innerR - distCenter;
    float distInner2 = innerR2 - distCenter;

    vec4 col = vec4(colors[0], getRingAlpha(uv, outerR, thicknessOuter, glow, .3, 1.5, 0.));
    col = alphaBlend(col, vec4(colors[0], getRingAlpha(uv, outerR, thicknessOuter, glow, .3, 1.5, 0.)));
    col = alphaBlend(col, vec4(colors[0], getRingAlpha(uv, innerR, 0., glow * 1.5, .5, 1.7, 1.) * innerAlpha));
    col = alphaBlend(col, vec4(colors[1], getRingAlpha(uv, innerR2, thicknessInner * .5, glow * 2., .9, 1., .5) * innerAlpha2));
    
    float heightShift = 0.3 + anim / 8.;
    const float timeShift = 0.4;
    for (float i = 2.; i >= -3.; i--)
    {
        float h = .35 + i * heightShift - (float(i <= -1.) * heightShift * 1.5) + sin(u_time + timeShift * i) * heightShift * (1. - anim);
        col = alphaBlend(col, vec4(colors[0], float(distOuter - thicknessOuter * .5 > 0.) * getAngleAlpha(uv, colors[0], h)));
    }

    float v = mix(0.5, 0.0, distance(uv,
        vec2(cos(u_time * 1.13 + sin(u_time * .5) * 2.),
            sin(u_time * 2.73)) * innerR * .8)) * float(distCenter < outerR + thicknessOuter * .5);
    col = alphaBlend(col, vec4(colors[0], v));

    gl_FragColor = col;
}
#endif