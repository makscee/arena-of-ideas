#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    v_quad_pos = a_pos;
    gl_Position = vec4(a_pos.xy, 0.0, 1.);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_previous_texture;
uniform ivec2 u_window_size;
in vec2 v_quad_pos;

uniform vec3 u_color_1 = vec3(0.01);
uniform vec3 u_color_2 = vec3(0.04);
uniform float cellSize = 150;

ivec2 cellIndex(vec2 uv)
{
    return ivec2(floor(uv.x), floor(uv.y));
}

vec2 cellCenter(ivec2 iuv)
{
    return iuv;
}

float distToLine(vec2 uv, float angle, float period)
{
    // uv.x += u_time;
    uv = rotateCW(uv, angle);
    float leftPeriod = floor(uv.x / period) * period;
    return min(uv.x - leftPeriod, leftPeriod + period - uv.x);
}

vec4 renderLine(vec2 uv, float clampBrightness, float width, float drop, float rotSpeed, float angleRange, float spreadness)
{
    uv += vec2(sin(u_time), cos(u_time)) * u_window_size.x / cellSize;
    return vec4(clampBrightness * smoothstep(drop, width, distToLine(uv, sin((u_time + 1000) * rotSpeed) * angleRange, spreadness)));
}

vec4 renderCell(vec2 uv)
{
    ivec2 iuv = cellIndex(uv);
    vec2 cuv = cellCenter(iuv);

    vec3 color1 = u_color_1;
    vec3 color2 = u_color_2;

    vec4 col = vec4(mix(color1, color2, float((iuv.x + iuv.y) % 2 == 0)), 1);
    // float dist = distance(cuv, vec2(cos(u_time), sin(u_time * 1.3)) * 900 + vec2(350)) / cellSize / 35;
    // dist = clamp(dist, 0, 1);
    // col = alphaBlend(col, vec4(0.3 * smoothstep(0.8, 0.0, dist)));
    float clampBrightness = 0.15;
    col = alphaBlend(col, renderLine(cuv, clampBrightness, 5, 12, 0.006, 33, 28));
    col = alphaBlend(col, renderLine(cuv, clampBrightness, 3, 5, 0.003, 60, 40));
    col = alphaBlend(col, renderLine(cuv, clampBrightness, 4, 2, 0.006, 60, 50));
    col = alphaBlend(col, renderLine(cuv, clampBrightness, 7, 12, 0.002, 180, 70));
    return col;
}

void main() {
    vec2 uv = (v_quad_pos * u_window_size + u_window_size / 2) / cellSize;
    vec4 col = renderCell(uv);
    gl_FragColor = col;
}
#endif