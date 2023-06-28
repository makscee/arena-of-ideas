#include <common.glsl>

uniform float u_head;
uniform float u_velocity;

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

vec4 stripes() {
    const float SPACING = 1.0;
    const float COUNT = 4;
    float y = -2.0;
    float sdf = 1.0;
    float offset = u_head - floor(u_head / SPACING);
    for(float i = -COUNT; i <= COUNT; i++) {
        float x = i * SPACING - offset;
        sdf = min(sdf, rectangle_sdf(uv - vec2(x, y), vec2(0.05, 0.5), 0));
    }
    return vec4(u_color.rgb, float(sdf < 0));
}

void main() {
    vec4 color = vec4(0);

    if(u_velocity != 0) {
        vec2 triangle_uv = uv;
        triangle_uv.x /= u_velocity;
        float sdf = triangle_sdf(triangle_uv, 1., -0.25);
        vec4 triangle = u_color;
        triangle.a = float(sdf < 0.);
        color = alpha_blend(color, triangle);
    }

    color = alpha_blend(color, stripes());

    gl_FragColor = color;
}
#endif
