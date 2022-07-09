#define CIRCLE 1
#define SQUARE 2
#define HEART 3

float p_distToShape(vec2 uv)
{
    if (p_shape == HEART)
    {
        return (uv.x * uv.x + uv.y * uv.y - 1) * (uv.x * uv.x + uv.y * uv.y - 1) * (uv.x * uv.x + uv.y * uv.y - 1) - uv.x * uv.x * uv.y * uv.y * uv.y;
    } else if (p_shape == CIRCLE)
    {
        return distance(uv,vec2(0)) - 1.;
    } else if (p_shape == SQUARE)
    {
        return max(uv.x - .5, uv.y - .5);
    } else
    {
        return 100500;
    }
}

#ifndef p_colorOverT_redef
vec3 p_colorOverT(int i, float t)
{
    return mix(p_startColor.rgb, p_endColor.rgb, t);
}
#endif

#ifndef p_radiusOverT_redef
float p_radiusOverT(int i, float t)
{
    return mix(p_startRadius, p_endRadius, t);
}
#endif

#ifndef p_alphaOverT_redef
float p_alphaOverT(int i, float t)
{
    return mix(p_startAlpha, p_endAlpha, t);
}
#endif

#ifndef p_startPosition_redef
vec2 p_startPosition(int i)
{
    return vec2(0);
}
#endif

#ifndef p_velocityOverT_redef
vec2 p_velocityOverT(int i, float t)
{
    return p_velocity;
}
#endif

#ifndef p_positionOverT_redef
vec2 p_positionOverT(int i, float t)
{
    return p_startPosition(i) + p_velocityOverT(i, t) * t;
}
#endif

#ifndef p_renderParticle_redef
vec4 p_renderParticle(int i, vec2 uv, float t)
{
    if (t < 0.) return vec4(0.);
    float radius = p_radiusOverT(i, t);
    if (radius < 0) return vec4(0);
    float distance = p_distToShape(uv);
    float alpha = float(distance < 0) * p_alphaOverT(i,t);
    return vec4(p_colorOverT(i,t), alpha);
}
#endif