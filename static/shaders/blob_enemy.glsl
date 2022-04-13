uniform float u_time;
uniform vec2 u_unit_position;
uniform float u_unit_radius;
uniform float u_spawn;
uniform float u_action;

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

const float pi = 3.14159;

float animationFunc(float x)
{
    const float t = 4.2;
    const float b = 2.1;
    return float(x < 0.769) * ((x * 1.3 - 1.) * (x * 1.3 - 1.) - 1.)
    - float(x > 0.769 && x < 1.)
    + float(x > 1. && x < 2.) * ((x * b - t) * (x * b - t));
}

vec3 getRingColor(
    vec2 uv, float r, float thickness, float glow, float glowStartV,
    vec3 colorRing, vec3 colorGlow, float innerMult, float outerMult)
{
    float dist = distance(uv, vec2(0.));
    float circleDist = abs(r - dist);
    float halfThickness = thickness * .5;
    glow *= max(r, 0.5);
    return float(circleDist < halfThickness) * colorRing
        + float(circleDist > halfThickness && circleDist < halfThickness + glow)
        * mix(glowStartV, 0., (circleDist - halfThickness) / glow)
        * (float(dist > r) * outerMult + float(dist < r) * 1.)
        * (float(dist < r) * innerMult + float(dist > r) * 1.)
        * colorGlow;
}

void main() {
    const float thicknessOuter = 0.07;
    const float thicknessInner = thicknessOuter * .5;
    float glow = 0.35 + sin(u_time) * .1;

    vec2 uv = v_quad_pos;

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
    float val = distCenter;

    vec3 col = vec3(0.0,0.0,0.0);
    col += getRingColor(uv, outerR, thicknessOuter, glow, .3, colors[0], colors[0], 1.5, 0.);
    col += getRingColor(uv, innerR, 0., glow * 1.5, .5, colors[0], colors[0], 1.7, 1.) * innerAlpha;
    col += getRingColor(uv, innerR2, thicknessInner * .5, glow * 2., .9, colors[1], colors[0], 1., .5) * innerAlpha2;
    
    float v = mix(0.5, 0.0, distance(uv, vec2(cos(u_time * 1.13 + sin(u_time * .5) * 2.), sin(u_time * 2.73)) * innerR * .8)) * float(distCenter < outerR + thicknessOuter * .5);
    col += float(v > 0.) * v * colors[0] * (1.5 - glow);

    // Output to screen
    gl_FragColor = vec4(col, 1. - distance(colors[0], col));
}
#endif