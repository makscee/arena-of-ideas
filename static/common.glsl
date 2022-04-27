const float pi = 3.14159;
const float thickness = 0.07;
const float glow = 3.;
const float glowStart = 0.8;
const float glowEnd = 0.1;
const float padding = 1.5;

const float thicknessOuter = 0.07;
const float thicknessInner = thicknessOuter * .5;

uniform float u_time;
uniform float u_injure_time;
uniform float u_spawn;
uniform float u_action;

uniform vec2 u_unit_position;
uniform float u_unit_radius;

uniform vec4 u_alliance_color_1;
uniform vec4 u_alliance_color_2;
uniform vec4 u_alliance_color_3;
uniform int u_alliance_count;

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
