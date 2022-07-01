

vec3 p_colorOverT(int i, float t)
{
    return mix(p_startColor.rgb, p_endColor.rgb, t);
}

float p_radiusOverT(int i, float t)
{
    return mix(p_startRadius, p_endRadius, t);
}

float p_alphaOverT(int i, float t)
{
    return mix(p_startAlpha, p_endAlpha, t);
}

vec2 p_positionOverT(int i, float t)
{
    return p_startPosition(i) + p_velocityOverT(i, t);
}