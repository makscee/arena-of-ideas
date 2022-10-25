vec4 shapeRender(vec2 uv, int index) {
    vec4 cols[3];
    cols[0] = vec4(0);
    cols[1] = getColor();
    cols[2] = vec4(0, 0, 0, 1);

    // shadow
    float dist = shapeDistance(uv - SHADOW_OFFSET, index, u_size - SHADOW_RADIUS * .5);
    vec4 col = vec4(0);
    if(dist < SHADOW_RADIUS) {
        float blur = smoothstep(SHADOW_RADIUS, 0., dist);
        blur = clamp(blur, 0., 1.);
        vec4 shadow = SHADOW_COLOR;
        shadow.a *= blur;
        col = shadow;
    }
    // shape
    dist = shapeDistance(uv, index, u_size);
    if(dist < 0) {
        if(dist > -u_outline_thickness)
            col = cols[u_outline];
        else
            col = cols[u_fill];
    }
    return col;
}