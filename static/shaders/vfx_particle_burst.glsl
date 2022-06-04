#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    const float padding = 1.;
    v_quad_pos = a_pos * (1.0 + padding);
    float size = u_unit_radius;
    vec2 pos = v_quad_pos * size + u_unit_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;

uniform float p_startSize;

const int p_trailCount = 4;
const float p_trailShift = 0.025;
const float p_spinMax = 0.1;
const float p_startPosRand = 0.1;
const float p_spawnShift = 0.02;
const float velocity = 0.6;

float p_velocityOverT(int i, float t)
{
    return e_invSquare(t) * velocity * (1. - rand(i + 1) * .2);
}

vec2 p_startPosition(int i)
{
    return randCircle(i) * rand(i) * p_startPosRand;
}

vec2 p_positionOverT(int i, float t)
{
    float spinRand = (0.5 + rand(i + 55)) * t * .7;
    return p_startPosition(i) + randCircle(i) * p_velocityOverT(i,t)
        + vec2(sin(spinRand * 17.), cos(spinRand * 13.)) * p_spinMax;
}

float p_radiusOverT(int i, float t)
{
    return mix(p_startSize, 0., t);
}

vec3 p_colorOverT(int i, float t)
{
    const int p_colorsNum = 2;

    vec3 p_colors[p_colorsNum];
    p_colors[0] = p_startColor.rgb;
    p_colors[1] = p_endColor.rgb;
    // p_colors[2] = vec3(1, 0.980, 0.941);
    // p_colors[3] = vec3(0.117, 0.564, 1);
    // p_colors[4] = vec3(0.117, 0.564, 1);
    t = clamp(t + ((rand(i + 3) - 0.5) * 0.3), 0., 1.);
    int colorInd = int(floor(t * float(p_colorsNum - 1)));
    return mix(p_colors[colorInd], p_colors[colorInd + 1], fract(t * float(p_colorsNum - 1)));
}

float p_alphaOverT(int i, float t)
{
    return mix(1., 0.6, t);
}

void p_discardCheck(vec2 uv, float t)
{
    if (distance(uv,vec2(0.)) > e_invSquare(t) * velocity + p_spinMax + p_startPosRand) discard;
}

vec4 p_renderParticle(int i, vec2 uv, float t)
{
    if (t < 0.) return vec4(0.);
    vec2 position = p_positionOverT(i, t);
    float radius = p_radiusOverT(i, t);
    float distance = distance(uv, position);
    float alpha = float(distance < radius) * p_alphaOverT(i,t);
    return vec4(p_colorOverT(i,t), alpha);
}

void main() {
    vec2 uv = v_quad_pos;
    float t = 1. - u_spawn;

    vec4 col = vec4(0,0,0,0);

    for (int i = 0; i < p_count; i++)
        for (int j = 0; j < p_trailCount; j++)
            {
                float sShift = float(i) * p_spawnShift;
                col = alphaBlend(col, p_renderParticle(i, uv, t / (1. - sShift) - float(j) * p_trailShift - sShift));
            }

    gl_FragColor = col;
}
#endif