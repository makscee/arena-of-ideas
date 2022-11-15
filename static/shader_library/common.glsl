const float pi = 3.14159;

const float ACTION_ANIMATION_TIME = 0.5;

const float c_thickness = .1;
const float c_glowRange = .2;
const float c_glowStart = 0.7;
const float c_glowEnd = 0.0;

const float injureAnimationTime = 0.5;

const float thicknessOuter = 0.07;
const float thicknessInner = thicknessOuter * .5;

const vec3 player_faction_color = vec3(0);
const vec3 enemy_faction_color = vec3(0.988, 0.004, 0.027);
const vec3 heal_color = vec3(0.129, 1, 0.024);

const vec2 SHADOW_OFFSET = vec2(-0.05, -0.05);
const vec4 SHADOW_COLOR = vec4(vec3(0), 0.5);
const float SHADOW_RADIUS = 0.1;

uniform float u_time;
uniform float u_injure_time;
uniform float u_heal_time;
uniform float u_spawn;
uniform float u_action = 0; // 0 -> 1
uniform float u_action_time;
uniform float u_cooldown;
uniform float u_ability_ready = 1;
uniform float u_random;
uniform float u_padding = 2;
uniform float u_health = 1;

uniform vec2 u_unit_position;
uniform vec2 u_face_dir;
uniform float u_unit_radius = 1;
uniform float u_ability_on_cooldown;

uniform vec2 u_parent_position;
uniform vec2 u_partner_position;
uniform float u_parent_radius;
uniform float u_parent_random;
uniform float u_parent_faction = 1;

uniform float u_thickness = 0.2;
uniform float u_curvature = 2;
uniform vec2 u_direction = vec2(0, 1);

uniform vec4 u_color = vec4(0);
uniform vec4 u_clan_color_1 = vec4(0.250, 0, 0.501, 1);
uniform vec4 u_clan_color_2 = vec4(0.117, 0.564, 1, 1);
uniform vec4 u_clan_color_3 = vec4(0.501, 0, 0.250, 1);
uniform int u_clan_count = 3;

uniform int u_status_count;
uniform int u_status_index;
uniform float u_status_time;
uniform float u_status_duration;
uniform vec4 u_status_color;

float clanCountF;
vec3 colors[3];
vec3 parent_faction_color;
vec3 parent_enemy_faction_color;

void commonInit() {
    colors[0] = u_clan_color_1.rgb;
    colors[1] = u_clan_color_2.rgb * float(u_clan_count > 1);
    colors[2] = u_clan_color_3.rgb * float(u_clan_count > 2);
    clanCountF = float(u_clan_count);
    parent_faction_color = mix(enemy_faction_color, player_faction_color, (u_parent_faction + 1) / 2);
    parent_enemy_faction_color = mix(enemy_faction_color, player_faction_color, 1 - (u_parent_faction + 1) / 2);
}

vec4 getColor() {
    return mix(vec4(parent_faction_color, 1), u_color, float(length(u_color.rgb) > 0));
}

vec4 alphaBlend(vec4 c1, vec4 c2) {
    return vec4(mix(c1.rgb, c2.rgb, c2.a), clamp(max(c1.a, c2.a) + c1.a * c2.a, 0., 1.));
}

float luminance(vec4 color) {
    return 0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b;
}

float animationFunc(float x) {
    const float t = 4.2;
    const float b = 2.1;
    return float(x < 0.769) * ((x * 1.3 - 1.) * (x * 1.3 - 1.) - 1.) - float(x > 0.769 && x < 1.) + float(x > 1. && x < 2.) * ((x * b - t) * (x * b - t));
}

int mod(int a, int b) {
    return a - (b * int(floor(float(a) / float(b))));
}

vec2 N22(vec2 p) {
    vec3 a = fract(p.xyx * vec3(123.34, 234.34, 345.65));
    a += dot(a, a + 34.45);
    return fract(vec2(a.x * a.y, a.y * a.z));
}

float rand(int i) {
    return N22(vec2(i * .001)).x;
}

vec2 randVec(int i) {
    return N22(vec2(i * .001));
}

vec2 randCircle(int i) {
    float r2p = rand(i) * pi * 2.;
    return vec2(cos(r2p), sin(r2p));
}

float invSquare(float t) {
    return 1. - (t - 1.) * (t - 1.);
}

vec2 rotateCW(vec2 p, float a) {
    mat2 m = mat2(cos(a), -sin(a), sin(a), cos(a));
    return p * m;
}

float vecAngle(vec2 v) {
    if(v == vec2(0.))
        return 0.;
    float r = acos(dot(normalize(v), vec2(0., 1.)));
    return (r + float(v.x > 0.) * (pi - r) * 2.);
}

float glowValue(float t) {
    return mix(c_glowStart, c_glowEnd, t);
}

float triangleDist(vec2 p, float radius) {
    return max(abs(p).x * 0.866025 + p.y * 0.5, -p.y) - radius * 0.5;
}

vec3 mixColors(float t) {
    t += float(t < 0.);
    int colorInd = int(t * clanCountF);
    vec3 c1 = colors[colorInd];
    vec3 c2 = colors[(colorInd + 1) % u_clan_count];
    return mix(c1, c2, t * clanCountF - float(colorInd));
}

vec3 mix3Colors(float t, vec3 colors[3]) {
    t += float(t < 0.);
    int colorInd = int(t * 3);
    vec3 c1 = colors[colorInd];
    vec3 c2 = colors[(colorInd + 1) % 3];
    return mix(c1, c2, t * 3 - float(colorInd));
}

vec3 mix2Colors(float t, vec3 colors[2]) {
    t += float(t < 0.);
    int colorInd = int(t * 2);
    vec3 c1 = colors[colorInd];
    vec3 c2 = colors[(colorInd + 1) % 3];
    return mix(c1, c2, t * 2 - float(colorInd));
}

vec3 hueShift(vec3 color, float hueAdjust) // hue in radians
{
    const vec3 kRGBToYPrime = vec3(0.299, 0.587, 0.114);
    const vec3 kRGBToI = vec3(0.596, -0.275, -0.321);
    const vec3 kRGBToQ = vec3(0.212, -0.523, 0.311);

    const vec3 kYIQToR = vec3(1.0, 0.956, 0.621);
    const vec3 kYIQToG = vec3(1.0, -0.272, -0.647);
    const vec3 kYIQToB = vec3(1.0, -1.107, 1.704);

    float YPrime = dot(color, kRGBToYPrime);
    float I = dot(color, kRGBToI);
    float Q = dot(color, kRGBToQ);
    float hue = atan(Q, I);
    float chroma = sqrt(I * I + Q * Q);

    hue += hueAdjust;

    Q = chroma * sin(hue);
    I = chroma * cos(hue);

    vec3 yIQ = vec3(YPrime, I, Q);

    return vec3(dot(yIQ, kYIQToR), dot(yIQ, kYIQToG), dot(yIQ, kYIQToB));
}

float smoothhump(float left, float right, float t) // 0 -> 1 -> 0
{
    return min(smoothstep(0., left, t), smoothstep(1., right, t));
}

vec2 toBezier(float t, vec2 P0, vec2 P1, vec2 P2, vec2 P3) {
    float t2 = t * t;
    float one_minus_t = 1.0 - t;
    float one_minus_t2 = one_minus_t * one_minus_t;
    return (P0 * one_minus_t2 * one_minus_t + P1 * 3.0 * t * one_minus_t2 + P2 * 3.0 * t2 * one_minus_t + P3 * t2 * t);
}

vec2 toBezierNormal(float t, vec2 P0, vec2 P1, vec2 P2, vec2 P3) {
    float t2 = t * t;
    vec2 tangent = P0 * (-3 * t2 + 6 * t - 3) +
        P1 * (9 * t2 - 12 * t + 3) +
        P2 * (-9 * t2 + 6 * t) +
        P3 * (3 * t2);
    return normalize(vec2(tangent.y, -tangent.x));
}

vec4 bezierParentPartner(float t, vec2 parent, vec2 partner) {
    // vec2 dir = normalize(parent - partner);
    vec2 dir = u_direction * u_curvature * (1 + (u_parent_random - 0.5) * .3);
    vec2 p0 = parent;
    vec2 p1 = parent + dir;
    vec2 p2 = partner + dir;
    vec2 p3 = partner;
    return vec4(toBezier(t, p0, p1, p2, p3), toBezierNormal(t, p0, p1, p2, p3));
}

float clanColorHash() {
    float h = colors[0].r * 1.1 + colors[0].g * 2.1 + colors[0].b * 4.3;
    h += colors[1].r * 1.1 + colors[1].g * 2.1 + colors[1].b * 4.3;
    h += colors[2].r * 1.1 + colors[2].g * 2.1 + colors[2].b * 4.3;
    return fract(h);
}

float colorHash(vec3 color) {
    return fract(color.r * 1.1 + color.g * 2.1 + color.b * 4.3);
}

/// Shapes
float distance1d(float x, float size) {
    return max(-size - x, x - size);
}

float toPoint(vec2 p, vec2 o) {
    return length(p - o);
}

float toSegment(vec2 p, vec2 a, vec2 b) {
    vec2 v = normalize(b - a);
    vec2 n = vec2(v.y, -v.x);
    return dot(p - a, n);
}

float triangleSDF(vec2 uv, float size, float rotation) {
    uv = rotateCW(uv, rotation * pi * 2);
    vec2 p = uv;
    vec2 p1 = -vec2(size);
    vec2 p2 = vec2(size, -size);
    vec2 p3 = vec2(0., size);
    float d1 = toSegment(p, p1, p2);
    float d2 = toSegment(p, p2, p3);
    float d3 = toSegment(p, p3, p1);
    float d = max(max(d1, d2), d3);
    if(dot(p - p1, p1 - p2) > 0.0 && dot(p - p1, p1 - p3) > 0.0) {
        d = toPoint(p, p1);
    }
    if(dot(p - p2, p2 - p1) > 0.0 && dot(p - p2, p2 - p3) > 0.0) {
        d = toPoint(p, p2);
    }
    if(dot(p - p3, p3 - p1) > 0.0 && dot(p - p3, p3 - p2) > 0.0) {
        d = toPoint(p, p3);
    }
    return d;
}

float squareSDF(vec2 uv, float size, float rotation) {
    uv = rotateCW(uv, rotation * pi * 2);
    float dx = distance1d(uv.x, size);
    float dy = distance1d(uv.y, size);

    float d = max(dx, dy);
    if(sign(dx) > 0.0 && sign(dy) > 0.0) {
        float corner_distance = sqrt(dx * dx + dy * dy);
        d = max(d, corner_distance);
    }
    return d;
}

float squareSDF(vec2 uv, float size) {
    return squareSDF(uv, size, 0.);
}

float circleSDF(vec2 uv, float radius) {
    return length(uv) - radius;
}
