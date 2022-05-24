#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    v_quad_pos = a_pos * (1.0 + u_padding);
    float size = u_unit_radius;
    vec2 pos = v_quad_pos * size + u_unit_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER

#define uv v_quad_pos
#define t u_time

in vec2 v_quad_pos;

uniform int p_count = 80;
uniform float p_life_time = 0.08;
uniform float p_radius_start = 0.2;
uniform float p_radius_end = 0.00;
uniform float p_glow_radius = 0.3;
uniform float p_speed = 3;
uniform vec2 u_velocity;


float p_spread(int i, int offset)
{
    return rand(i + offset) - 0.5;
}

float p_lifeT(int i, float t)
{
    float maxLife = p_life_time + p_spread(i, 1) * 1. * p_life_time;
    float ut = (t - floor(t / maxLife) * maxLife) / maxLife;
    float lifeT = ut - i / float(p_count);
    lifeT += float(lifeT < 0);
    return lifeT;
}

vec2 p_positionOverT(int i, float t)
{
    float speed = p_speed + p_speed * p_speed * .8;

    float vSpread = p_spread(i, 3) * .6;
    vec2 velocity = -rotateCW(normalize(u_velocity) * speed, vSpread);
    return velocity * p_lifeT(i, t);
}

vec3 p_color(int i)
{
    return mix3Colors(rand(i), colors);
}

float p_radiusOverT(int i, float t)
{
    return mix(p_radius_start, p_radius_end, p_lifeT(i, t));
}

float p_glow(float dist, float radius)
{
    return smoothstep(p_glow_radius, 0.1, dist - radius) * .3;
}

vec4 p_renderParticle(vec2 uv, int i, float t)
{
    vec2 position = p_positionOverT(i, t);
    float dist = distance(uv, position);
    float radius = p_radiusOverT(i, t);
    float alpha = max(float(dist < radius), p_glow(dist, radius));
    return vec4(p_color(i), alpha);
}

bool p_discardCheck(vec2 uv, float t)
{
    return length(uv) > p_speed * 2.;
}

void main() {
    if (p_discardCheck(uv, t))
    {
        gl_FragColor = vec4(0);
        return;
    }
    commonInit();

    vec4 col = vec4(0);
    for (int i = 0; i < p_count; i++)
        col = alphaBlend(col, p_renderParticle(uv, i, t));

    gl_FragColor = col;
}
#endif