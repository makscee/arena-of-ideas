uniform vec4 u_color;
uniform int u_faction = -1;
uniform float u_game_time;

vec3 faction_color;

void commonInit() {
    faction_color = mix(vec3(0), vec3(1), (1. + float(u_faction)) * .5);
}

// vec4 getColor() {
//     return mix(vec4(parent_faction_color, 1), u_color, float(length(u_color.rgb) > 0));
// }

vec4 alphaBlend(vec4 c1, vec4 c2) {
    return vec4(mix(c1.rgb, c2.rgb, c2.a), clamp(max(c1.a, c2.a) + c1.a * c2.a, 0., 1.));
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

float squareSDF(vec2 uv, float size, float rotation) {
    uv = rotateCW(uv, rotation * PI * 2);
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
