const float pi = 3.14159;

const float c_thickness = .1;
const float c_glowRange = .2;
const float c_glowStart = 0.7;
const float c_glowEnd = 0.0;

const float c_status_radius_delta = .2;
const float c_status_radius_delta_max = .75;
const float c_status_radius_offset = .3;
const float c_status_thickness = .025;
const float c_status_dot_radius = 0.09;

const float padding = 1.5;

const float thicknessOuter = 0.07;
const float thicknessInner = thicknessOuter * .5;

uniform float u_time;
uniform float u_injure_time;
uniform float u_spawn;
uniform float u_action; // 0 -> 1
uniform float u_action_time;
uniform float u_animation_delay;
uniform float u_cooldown;
uniform float u_ability_ready;
uniform float u_random;
uniform float u_padding;
uniform float u_health;

uniform vec2 u_unit_position;
uniform vec2 u_face_dir;
uniform float u_unit_radius;
uniform float u_ability_on_cooldown;

uniform vec4 u_clan_color_1;
uniform vec4 u_clan_color_2;
uniform vec4 u_clan_color_3;
uniform int u_clan_count;

uniform int u_status_count;
uniform int u_status_index;
uniform vec4 u_status_color;

float alCountF;
vec3 colors[3];

void commonInit()
{
    colors[0] = u_clan_color_1.rgb;
    colors[1] = u_clan_color_2.rgb;
    colors[2] = u_clan_color_3.rgb;
    alCountF = float(u_clan_count);
}

vec4 alphaBlend(vec4 c1, vec4 c2)
{
    return vec4(
        mix(c1.r, c2.r, c2.a),
        mix(c1.g, c2.g, c2.a),
        mix(c1.b, c2.b, c2.a),
        clamp(max(c1.a, c2.a) + c1.a * c2.a, 0., 1.));
}

float animationFunc(float x)
{
    const float t = 4.2;
    const float b = 2.1;
    return float(x < 0.769) * ((x * 1.3 - 1.) * (x * 1.3 - 1.) - 1.)
    - float(x > 0.769 && x < 1.)
    + float(x > 1. && x < 2.) * ((x * b - t) * (x * b - t));
}

int mod(int a, int b)
{
    return a - (b * int(floor(float(a)/float(b))));
}

vec2 N22(vec2 p) 
{
  vec3 a = fract(p.xyx*vec3(123.34, 234.34, 345.65));
  a += dot(a, a+34.45);
  return fract(vec2(a.x*a.y, a.y*a.z));
}

float rand(int i)
{
    return N22(vec2(i)).x;
}

vec2 randCircle(int i) 
{
    float r2p = rand(i) * pi * 2.;
    return vec2(cos(r2p), sin(r2p));
}

float e_invSquare(float t)
{
    return 1. - (t - 1.) * (t - 1.);
}

vec2 rotateCW(vec2 p, float a)
{
    mat2 m = mat2(cos(a), -sin(a), sin(a), cos(a));
    return p * m;
}

float vecAngle(vec2 v)
{
    if (v == vec2(0.)) return 0.;
    float r = acos(dot(normalize(v), vec2(0.,1.)));
    return (r + float(v.x > 0.) * (pi - r) * 2.);
}

float glowValue(float t)
{
    return mix(c_glowStart, c_glowEnd, t);
}

float triangleDist(vec2 p, float radius)
{
    return max(abs(p).x * 0.866025 + p.y * 0.5, -p.y) - radius * 0.5;
}

vec3 mixColors(float t)
{
    t += float(t < 0.);
    int colorInd = int(t * alCountF);
    vec3 c1 = colors[colorInd];
    vec3 c2 = colors[(colorInd + 1) % u_clan_count];
    return mix(c1, c2, t * alCountF - float(colorInd));
}

vec3 mix3Colors(float t, vec3 colors[3])
{
    t += float(t < 0.);
    int colorInd = int(t * 3);
    vec3 c1 = colors[colorInd];
    vec3 c2 = colors[(colorInd + 1) % 3];
    return mix(c1, c2, t * 3 - float(colorInd));
}

vec3 mix2Colors(float t, vec3 colors[2])
{
    t += float(t < 0.);
    int colorInd = int(t * 2);
    vec3 c1 = colors[colorInd];
    vec3 c2 = colors[(colorInd + 1) % 3];
    return mix(c1, c2, t * 2 - float(colorInd));
}

vec4 renderStatusRing(vec2 uv, vec3 col)
{
    const float u_status_duration = 0.;
    // float u_status_time = 0.8 + sin(iTime) * .5;
    const float u_status_time = 0.;

    float offset = 1. + c_status_radius_offset + c_status_radius_delta * u_status_index
        * (min(1., c_status_radius_delta_max / c_status_radius_delta / u_status_count));
    float rad = abs(vecAngle(uv) - pi);
    float h = abs(distance(uv,vec2(0.)) - offset);
    float dotDistance = distance(uv, vec2(0,-1) * offset);
    return vec4(col, 
        float(h < c_status_thickness && (u_status_duration == 0. || rad < u_status_time / u_status_duration * pi)
        || dotDistance < c_status_dot_radius));
}

float smoothhump(float left, float right, float t)
{
    return min(smoothstep(0.,left,t), smoothstep(1.,right,t));
}




// particles


uniform vec4 p_startColor = vec4(0.862, 0.078, 0.235, 1);
uniform vec4 p_endColor = vec4(0.117, 0.564, 1, 1);
uniform float p_lifeTime = 2;
uniform float p_radius = 0.1;
uniform vec2 p_velocity = vec2(0.05,0.3);
uniform int p_count = 10;
uniform int p_shape = 0;

#ifdef FRAGMENT_SHADER

#define CIRCLE 1
#define HEART 2

float p_alphaOverT_def(int i, float t)
{
#ifndef p_alphaOverT
    return smoothhump(0.3,0.5,t);
#else
    return p_alphaOverT(i,t);
#endif
}

float p_radiusOverT_def(int i, float t)
{
#ifndef p_radiusOverT
    return smoothhump(0.5,0.99,t) * p_radius;
#else
    p_radiusOverT(i,t);
#endif
}

vec3 p_colorOverT_def(int i, float t)
{
#ifndef p_colorOverT
    return mix(p_startColor.rgb, p_endColor.rgb, t);
#else
    return p_colorOverT(i,t);
#endif
}

float p_distToShape(vec2 pos, vec2 uv, float radius)
{
    if (p_shape == HEART)
    {
        uv -= pos;
        uv /= radius;
        return (uv.x * uv.x + uv.y * uv.y - 1) * (uv.x * uv.x + uv.y * uv.y - 1) * (uv.x * uv.x + uv.y * uv.y - 1) - uv.x * uv.x * uv.y * uv.y * uv.y;
    } else if (p_shape == CIRCLE)
    {
        return distance(pos, uv) - radius;
    } else
    {
        return 100500;
    }
}

vec2 p_positionOverT_def(int i, float t)
{
#ifndef p_positionOverT
    vec2 velocity = rotateCW(p_velocity, (rand(i+2) - 0.5) * pi / 4);
    vec2 startPos = randCircle(i) * rand(i+1);
    return mix(startPos, startPos + velocity * p_lifeTime , t);
#else
    return p_positionOverT(i,t);
#endif
}

void p_discardCheck_def(vec2 uv, float t)
{
#ifndef p_discardCheck
    if (uv.y < -2.) discard;
#else
    p_discardCheck(uv, t);
#endif
}

#endif