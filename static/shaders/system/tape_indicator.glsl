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

uniform vec4 u_color_2;

vec4 stripes(float offset) {
    const float SPACING = 1;
    const float COUNT = 4;
    float y = -2.0;
    float sdf = 1.0;
    offset = u_head - floor(u_head / SPACING) + offset;
    for(float i = -COUNT; i <= COUNT; i++) {
        float x = i * SPACING - offset;
        sdf = min(sdf, rectangle_sdf(uv - vec2(x, y), vec2(0.05, 0.5), 0));
    }
    return vec4(u_color.rgb, float(sdf < 0));
}

void main() {
    vec4 color = vec4(0);
    float offset = 0;
    float vel = u_velocity + offset * 0.2;
    if(vel != 0) {
        vec4 bg_color = u_color_2;
        vec2 triangle_uv = uv;
        float sdf = triangle_sdf(triangle_uv, 1., -0.5);
        bg_color.a = float(sdf < 0.);
        color = alpha_blend(color, bg_color);
        vec4 main_color = u_color;
        triangle_uv.x = (triangle_uv.x + .5) / vel - .5;
        sdf = triangle_sdf(triangle_uv, 1., -0.5);
        main_color.a = float(sdf < 0.);
        color = alpha_blend(color, main_color);
    }

    color = alpha_blend(color, stripes(offset * 0.05));

    gl_FragColor = color;
}
#endif
