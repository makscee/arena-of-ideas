uniform vec4 u_color = vec4(1, 0, 1, 1);
uniform vec4 u_house_color;
uniform float u_faction = 1;
uniform float u_game_time;
uniform float u_global_time;
uniform vec2 u_field_position;
uniform float u_t = 0;

uniform vec4 u_outline_color;
uniform vec4 u_background_light;
uniform vec4 u_background_dark;
vec3 base_color;
vec3 field_color;

uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

uniform float u_card = 0;
float card;
uniform float u_zoom = 1; // only for internal use
float zoom;
uniform vec2 u_position = vec2(0);
vec2 position;
uniform vec2 u_offset = vec2(0, 0);
uniform vec2 u_box = vec2(0);
vec2 box;
uniform float u_radius = 0;
float radius;
uniform float u_padding = 0;
float padding;
uniform float u_scale = 1;
uniform float u_size = 1;
uniform float u_rotation = 0;
float rotation;
uniform int u_index;
uniform float u_rand;
uniform float u_aspect_ratio;

uniform int u_ui = 0;

uniform int u_sdf_gradient_points = 0;
uniform vec4 u_g_points;
uniform vec4 u_g_alphas = vec4(1);
uniform vec4 u_g_color_1;
uniform vec4 u_g_color_2;
uniform vec4 u_g_color_3;
uniform vec4 u_g_color_4;

uniform float u_warp_str = 0;
uniform float u_warp_speed = 1;
uniform float u_fbm_sdf = 0;
uniform float u_fbm_sdf_size = 1.0;
uniform float u_fbm_sdf_speed = 1;

vec4 sdf_gradient(float x) {
    float oob = 1. - float(x > u_g_points[3]);
    vec4 g1 = vec4(u_g_color_1.rgb, u_g_color_1.a * u_g_alphas[0]);
    vec4 g2 = vec4(u_g_color_2.rgb, u_g_color_2.a * u_g_alphas[1]);
    vec4 g3 = vec4(u_g_color_3.rgb, u_g_color_3.a * u_g_alphas[2]);
    vec4 g4 = vec4(u_g_color_4.rgb, u_g_color_4.a * u_g_alphas[3]);
    return oob * (float(x < u_g_points[0]) * u_color +
        float(x > u_g_points[0] && x < u_g_points[1]) * mix(g1, g2, (x - u_g_points[0]) / (u_g_points[1] - u_g_points[0])) +
        float(x > u_g_points[1] && x < u_g_points[2]) * mix(g2, g3, (x - u_g_points[1]) / (u_g_points[2] - u_g_points[1])) +
        float(x > u_g_points[2] && x < u_g_points[3]) * mix(g3, g4, (x - u_g_points[2]) / (u_g_points[3] - u_g_points[2])));
}

vec2 get_card_uv(vec2 uv) {
    return mix(uv, uv * 2 + vec2(0, -.7), card);
}

vec2 get_card_pos(vec2 pos) {
    return mix(pos, (pos + vec2(0, .7)) / 2, card);
}

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

mat2 m = mat2(0.6, 0.6, -0.6, 0.8);
float fbm(vec2 p) {
    float f = 0.0;
    p += vec2(u_rand * 3., u_rand * 5.);
    f += 0.5000 * noise(p);
    p *= m * 2.02;
    f += 0.2500 * noise(p);
    p *= m * 2.03;
    f += 0.1250 * noise(p);
    p *= m * 2.01;
    f += 0.0625 * noise(p);
    p *= m * 2.04;
    f /= 0.9375;
    return f * 2. - 1.;
}

vec2 warp(vec2 uv, float t) {
    t *= u_warp_speed;
    vec2 q = vec2(fbm(uv), fbm(uv + vec2(1)));
    vec2 r = vec2(0);
    r.x = fbm(uv + q + vec2(1.1, 4.3) + t * 0.15);
    r.y = fbm(uv + q + vec2(8.3, 2.1) + t * 0.125);
    return uv + r * u_warp_str;
}

vec2 rotate_cw(vec2 p, float a) {
    mat2 m = mat2(cos(a), sin(a), -sin(a), cos(a));
    return p * m;
}

float fbm_sdf(float value, vec2 uv) {
    return value + (fbm(uv + rotate_cw(vec2(u_global_time * u_fbm_sdf_speed, 0.0), fbm(uv) * .05 * u_fbm_sdf_size)) - 0.5) * u_fbm_sdf;
}

float get_field_value(vec2 uv) {
    vec2 position = position + uv + u_field_position;
    return smoothstep(-0.1, 0.1, position.y * .4 - position.x + (fbm(position.yy + vec2(u_game_time * 0.3, 0)) - .5) * 2.);
}

void init_fields() {
    card = u_card;
    position = u_position;
    box = u_box;
    rotation = u_rotation;
    padding = u_padding;
    float field = get_field_value(position);
    base_color = mix(u_background_light.rgb, u_background_dark.rgb, field);
    field_color = mix(u_background_light.rgb, u_background_dark.rgb, 1 - field);
}

vec2 get_uv(vec2 a_pos) {
    return a_pos * (1.0 + padding);
}

vec4 get_gl_position(vec2 uv) {
    vec2 pos = rotate_cw(uv * box, rotation) + position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    vec4 cam_pos = vec4(p_pos.xy, 0.0, p_pos.z);
    vec4 ui_pos = vec4(pos, 0.0, 1.0);
    return mix(cam_pos, ui_pos, float(u_ui));
}

const float TEXT_AA = 0.01;
vec4 get_text_color(float sdf, vec4 text_color, vec4 outline_color, float text_border, float text_inside) {
    return mix(mix(vec4(0), outline_color, smoothstep(text_border - TEXT_AA, text_border + TEXT_AA, sdf)), text_color, smoothstep(text_inside - TEXT_AA, text_inside + TEXT_AA, sdf));
}

float get_text_sdf(vec2 uv, sampler2D sampler) {
    vec2 text_uv = uv * .5 + .5;
    return texture2D(sampler, text_uv).x;
}

vec4 alpha_blend(vec4 c1, vec4 c2) {
    return vec4(mix(c1.rgb, c2.rgb, c2.a), clamp(max(c1.a, c2.a), 0., 1.));
}

float luminance(vec4 color) {
    return 0.2126 * color.r + 0.7152 * color.g + 0.0722 * color.b;
}

vec2 n22(vec2 p) {
    vec3 a = fract(p.xyx * vec3(123.34, 234.34, 345.65));
    a += dot(a, a + 34.45);
    return fract(vec2(a.x * a.y, a.y * a.z));
}

float rand(int i) {
    return n22(vec2(i * .001)).x;
}

vec2 rand_vec(int i) {
    return n22(vec2(i * .001));
}

vec2 rand_circle(int i) {
    float r2p = rand(i) * PI * 2.;
    return vec2(cos(r2p), sin(r2p));
}

float inv_square(float t) {
    return 1. - (t - 1.) * (t - 1.);
}

float vec_angle(vec2 v) {
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

vec3 hue_shift(vec3 color, float hueAdjust) // hue in radians
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

float aliase(float left, float right, float smear, float t) {
    return min(smoothstep(left - smear, left + smear, t), smoothstep(right + smear, right - smear, t));
}

vec2 to_bezier(float t, vec2 P0, vec2 P1, vec2 P2, vec2 P3) {
    float t2 = t * t;
    float one_minus_t = 1.0 - t;
    float one_minus_t2 = one_minus_t * one_minus_t;
    return (P0 * one_minus_t2 * one_minus_t + P1 * 3.0 * t * one_minus_t2 + P2 * 3.0 * t2 * one_minus_t + P3 * t2 * t);
}

vec2 to_bezier_normal(float t, vec2 P0, vec2 P1, vec2 P2, vec2 P3) {
    float t2 = t * t;
    vec2 tangent = P0 * (-3 * t2 + 6 * t - 3) +
        P1 * (9 * t2 - 12 * t + 3) +
        P2 * (-9 * t2 + 6 * t) +
        P3 * (3 * t2);
    return normalize(vec2(tangent.y, -tangent.x));
}

vec4 bezier_parent_partner(float t, vec2 parent, vec2 partner, vec2 direction, float curvature) {
    // vec2 dir = normalize(parent - partner);
    vec2 dir = direction * curvature;
    vec2 p0 = parent;
    vec2 p1 = parent + dir;
    vec2 p2 = partner + dir;
    vec2 p3 = partner;
    return vec4(to_bezier(t, p0, p1, p2, p3), to_bezier_normal(t, p0, p1, p2, p3));
}

float color_hash(vec3 color) {
    return fract(color.r * 100.123 + color.g * 22.1512 + color.b * 420.6969);
}

/// Shapes
float distance1d(float x, float box) {
    return max(-box - x, x - box);
}

float to_point(vec2 p, vec2 o) {
    return length(p - o);
}

float to_segment(vec2 p, vec2 a, vec2 b) {
    vec2 v = normalize(b - a);
    vec2 n = vec2(v.y, -v.x);
    return dot(p - a, n);
}

float triangle_sdf(vec2 uv, float box, float rotation) {
    uv = rotate_cw(uv, rotation * PI * 2);
    vec2 p = uv;
    float angle = PI * 7 / 6;
    vec2 p1 = vec2(cos(angle), sin(angle)) * box;
    angle = -PI / 6;
    vec2 p2 = vec2(cos(angle), sin(angle)) * box;
    vec2 p3 = vec2(0., box);
    float d1 = to_segment(p, p1, p2);
    float d2 = to_segment(p, p2, p3);
    float d3 = to_segment(p, p3, p1);
    float d = max(max(d1, d2), d3);
    if(dot(p - p1, p1 - p2) > 0.0 && dot(p - p1, p1 - p3) > 0.0) {
        d = to_point(p, p1);
    }
    if(dot(p - p2, p2 - p1) > 0.0 && dot(p - p2, p2 - p3) > 0.0) {
        d = to_point(p, p2);
    }
    if(dot(p - p3, p3 - p1) > 0.0 && dot(p - p3, p3 - p2) > 0.0) {
        d = to_point(p, p3);
    }
    return d;
}

float rectangle_sdf(vec2 uv, vec2 box, float rotation) {
    uv = rotate_cw(uv, rotation * PI * 2);
    float dx = distance1d(uv.x, box.x);
    float dy = distance1d(uv.y, box.y);

    float d = max(dx, dy);
    if(sign(dx) > 0.0 && sign(dy) > 0.0) {
        float corner_distance = sqrt(dx * dx + dy * dy);
        d = max(d, corner_distance);
    }
    return d;
}

float rectangle_rounded_sdf(vec2 uv, vec2 box, vec4 r) {
    r.xy = (uv.x > 0.0) ? r.xy : r.zw;
    r.x = (uv.y > 0.0) ? r.x : r.y;
    vec2 q = abs(uv) - box + r.x;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - r.x;
}

float square_sdf(vec2 uv, float box, float rotation) {
    return rectangle_sdf(uv, vec2(box, box), rotation);
}

float square_sdf(vec2 uv, float box) {
    return square_sdf(uv, box, 0.);
}

float circle_sdf(vec2 uv, float radius) {
    return length(uv) - radius;
}