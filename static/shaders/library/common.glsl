uniform vec4 u_color;
uniform float u_faction = 1;
uniform float u_game_time;
uniform float u_global_time;
uniform vec2 u_field_position;

uniform float u_card;
uniform float u_hovered = 0;
uniform float u_is_battle;

vec3 light_color = vec3(1);
vec3 dark_color = vec3(0);
vec3 base_color;
vec3 field_color;

/// Noise
float hash(float n) {
    return fract(sin(n) * 75728.5453123);
}

float noise(in vec2 x) {
    vec2 p = floor(x);
    vec2 f = fract(x);
    f = f * f * (3.0 - 2.0 * f);
    float n = p.x + p.y * 57.0;
    return mix(mix(hash(n + 0.0), hash(n + 1.0), f.x), mix(hash(n + 57.0), hash(n + 58.0), f.x), f.y);
}

/// FBM
mat2 m = mat2(0.6, 0.6, -0.6, 0.8);
float fbm(vec2 p) {

    float f = 0.0;
    f += 0.5000 * noise(p);
    p *= m * 2.02;
    f += 0.2500 * noise(p);
    p *= m * 2.03;
    f += 0.1250 * noise(p);
    p *= m * 2.01;
    f += 0.0625 * noise(p);
    p *= m * 2.04;
    f /= 0.9375;
    return f;
}

float get_field_value(vec2 position) {
    position += u_field_position;
    return smoothstep(-0.1, 0.1, position.y * .4 - position.x + (fbm(position.yy + vec2(u_game_time * 0.3, 0)) - .5) * 2.);
}

const float TEXT_AA = 0.05;
vec4 get_text_color(float sdf, vec4 text_color, vec4 outline_color, float text_border, float text_inside) {
    return mix(mix(vec4(0), outline_color, smoothstep(text_border - TEXT_AA, text_border + TEXT_AA, sdf)), text_color, smoothstep(text_inside - TEXT_AA, text_inside + TEXT_AA, sdf));
}

float get_text_sdf(vec2 uv, sampler2D sampler) {
    vec2 text_uv = uv * .5 + .5;
    return texture2D(sampler, text_uv).x;
}

float get_card_value() {
    return mix(u_card, u_hovered, u_is_battle);
}

vec2 get_card_uv(vec2 uv, float card) {
    return mix(uv, uv * 2 + vec2(0, -.7), card);
}

vec2 get_card_pos(vec2 pos, float card) {
    return mix(pos, (pos + vec2(0, .7)) / 2, card);
}

void commonInit(vec2 position) {
    float field = get_field_value(position);
    base_color = mix(light_color, dark_color, field);
    field_color = mix(light_color, dark_color, 1 - field);
}

vec4 alphaBlend(vec4 c1, vec4 c2) {
    return vec4(mix(c1.rgb, c2.rgb, c2.a), clamp(max(c1.a, c2.a), 0., 1.));
}

float luminance(vec4 color) {
    return 0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b;
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
    float r2p = rand(i) * PI * 2.;
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
    return (r + float(v.x > 0.) * (PI - r) * 2.);
}

// vec3 mixColors(float t) {
//     t += float(t < 0.);
//     int colorInd = int(t * clanCountF);
//     vec3 c1 = colors[colorInd];
//     vec3 c2 = colors[(colorInd + 1) % u_clan_count];
//     return mix(c1, c2, t * clanCountF - float(colorInd));
// }

// vec3 mix3Colors(float t, vec3 colors[3]) {
//     t += float(t < 0.);
//     int colorInd = int(t * 3);
//     vec3 c1 = colors[colorInd];
//     vec3 c2 = colors[(colorInd + 1) % 3];
//     return mix(c1, c2, t * 3 - float(colorInd));
// }

// vec3 mix2Colors(float t, vec3 colors[2]) {
//     t += float(t < 0.);
//     int colorInd = int(t * 2);
//     vec3 c1 = colors[colorInd];
//     vec3 c2 = colors[(colorInd + 1) % 3];
//     return mix(c1, c2, t * 2 - float(colorInd));
// }

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

vec4 bezierParentPartner(float t, vec2 parent, vec2 partner, vec2 direction, float curvature) {
    // vec2 dir = normalize(parent - partner);
    vec2 dir = direction * curvature;
    vec2 p0 = parent;
    vec2 p1 = parent + dir;
    vec2 p2 = partner + dir;
    vec2 p3 = partner;
    return vec4(toBezier(t, p0, p1, p2, p3), toBezierNormal(t, p0, p1, p2, p3));
}

float colorHash(vec3 color) {
    return fract(color.r * 100.123 + color.g * 22.1512 + color.b * 420.6969);
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
    uv = rotateCW(uv, rotation * PI * 2);
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

float rectangle_sdf(vec2 uv, vec2 size, float rotation) {
    uv = rotateCW(uv, rotation * PI * 2);
    float dx = distance1d(uv.x, size.x);
    float dy = distance1d(uv.y, size.y);

    float d = max(dx, dy);
    if(sign(dx) > 0.0 && sign(dy) > 0.0) {
        float corner_distance = sqrt(dx * dx + dy * dy);
        d = max(d, corner_distance);
    }
    return d;
}

float squareSDF(vec2 uv, float size, float rotation) {
    return rectangle_sdf(uv, vec2(size, size), rotation);
}

float squareSDF(vec2 uv, float size) {
    return squareSDF(uv, size, 0.);
}

float circleSDF(vec2 uv, float radius) {
    return length(uv) - radius;
}