#include <common.glsl>
#ifdef VERTEX_SHADER
out vec2 uv;
attribute vec2 a_pos;

void main() {
    init_fields();
    uv = get_uv(a_pos);
    gl_Position = get_gl_position(uv);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 uv;

float sdf_star5(in vec2 p, in float r, in float rf) {
    const vec2 k1 = vec2(0.809016994375, -0.587785252292);
    const vec2 k2 = vec2(-k1.x, k1.y);
    p.x = abs(p.x);
    p -= 2.0 * max(dot(k1, p), 0.0) * k1;
    p -= 2.0 * max(dot(k2, p), 0.0) * k2;
    p.x = abs(p.x);
    p.y -= r;
    vec2 ba = rf * vec2(-k1.y, k1.x) - vec2(0, 1);
    float h = clamp(dot(p, ba) / dot(ba, ba), 0.0, r);
    return length(p - ba * h) * sign(p.y * ba.x - p.x * ba.y);
}

uniform float u_alpha = 1;
uniform float u_outline = 0;

void main() {
    float sdf = sdf_star5(uv, 1, 0.5);
    vec4 color = vec4(u_color.rgb, float(sdf < 0));
    vec4 outline_color = vec4(u_outline_color.rgb, float(sdf > -u_outline));
    color = alpha_blend(color, outline_color);
    color.a = float(sdf < 0) * u_alpha;
    gl_FragColor = color;
}
#endif
