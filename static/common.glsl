const float pi = 3.14159;

const float c_thickness = .1;
const float c_glowRange = .2;
const float c_glowStart = 0.7;
const float c_glowEnd = 0.0;

const float c_status_radius_delta = .2;
const float c_status_radius_offset = .4;
const float c_status_thickness = .035;
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
uniform float u_random;

uniform vec2 u_unit_position;
uniform vec2 u_face_dir;
uniform float u_unit_radius;

uniform vec4 u_alliance_color_1;
uniform vec4 u_alliance_color_2;
uniform vec4 u_alliance_color_3;
uniform int u_alliance_count;

uniform int u_status_count;
uniform int u_status_index;
uniform vec4 u_status_color;

float alCountF;
vec3 colors[3];

void commonInit()
{
    colors[0] = u_alliance_color_1.rgb;
    colors[1] = u_alliance_color_2.rgb;
    colors[2] = u_alliance_color_3.rgb;
    alCountF = float(u_alliance_count);
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

float round(float v)
{
	return floor(v) + float(v - floor(v) > 0.5) * 1.;
}

float rand(int i)
{
    return fract(sin(dot(vec2(i,0.) ,vec2(12.9898,78.233))) * 43758.5453);
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
    vec3 c2 = colors[(colorInd + 1) % u_alliance_count];
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

    float offset = 1. + c_status_radius_offset + c_status_radius_delta * u_status_index;
    float rad = abs(vecAngle(uv) - pi);
    float h = abs(distance(uv,vec2(0.)) - offset);
    float dotDistance = distance(uv, vec2(0,-1) * offset);
    return vec4(col, 
        float(h < c_status_thickness && (u_status_duration == 0. || rad < u_status_time / u_status_duration * pi)
        || dotDistance < c_status_dot_radius));
}
