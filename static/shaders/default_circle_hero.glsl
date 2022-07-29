#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
uniform float u_hole_radius = 0.1;

void main() {
    float action_t = smoothstep(ACTION_ANIMATION_TIME, 0, u_time - u_action_time);
    action_t *= action_t;
    float u_hole_radius = u_hole_radius + action_t * 0.2;
    float radius = (u_unit_radius - u_hole_radius) * UNITS_SCALE * (1 + action_t * .1);
    float height = u_hole_radius + radius * .5 + (radius * .5 * a_pos.y);
    float radian = a_pos.x * pi * 2.;
    float paddingHeight = float(a_pos.y > 0) * u_padding * radius;
    height += paddingHeight;
    v_quad_pos = vec2(a_pos.x, (a_pos.y + 1.) * .5 * (radius + paddingHeight) / radius);
    vec2 pos = vec2(sin(radian), cos(radian)) * height + u_unit_position;
    vec3 p_pos = u_projection_matrix * u_view_matrix * vec3(pos, 1.0);
    gl_Position = vec4(p_pos.xy, 0.0, p_pos.z);
}
#endif

#ifdef FRAGMENT_SHADER
in vec2 v_quad_pos;

vec3 getMixedClanColor(float t) {
    vec3 col = colors[0];
    if (u_clan_count == 2) {
        col = mix2Colors(t, vec3[2](colors[0],colors[1]));
    } else if (u_clan_count == 3) {
        col = mix3Colors(t, colors);
    }
    return col;
}

// Noise
float hash( float n ) {
    return fract(sin(n)*75728.5453123); 
}

float noise( in vec2 x ) {
    vec2 p = floor(x);
    vec2 f = fract(x);
    f = f*f*(3.0-2.0*f);
    float n = p.x + p.y*57.0;
    return mix(mix( hash(n + 0.0), hash(n + 1.0), f.x), mix(hash(n + 57.0), hash(n + 58.0), f.x), f.y);
}

// FBM
mat2 m = mat2( 0.6, 0.6, -0.6, 0.8);
float fbm(vec2 p){
 
    float f = 0.0;
    f += 0.5000 * noise(p); p *= m * 2.02;
    f += 0.2500 * noise(p); p *= m * 2.03;
    f += 0.1250 * noise(p); p *= m * 2.01;
    f += 0.0625 * noise(p); p *= m * 2.04;
    f /= 0.9375;
    return f;
}

vec3 stepClanColor(in float t) {
    vec3[4] c;
    for (int i = 0; i < u_clan_count; i++) {
        c[i] = colors[i];
    }
    c[u_clan_count] = parent_faction_color;
    int ind = int(floor(fract(t + u_time * .3) * (u_clan_count + 1)));
    float shift = floor(fract(t * u_clan_count + sin(u_time * .5)) / .333 - 1.);
    // shift = 0;
    if (ind == u_clan_count) return c[u_clan_count];
    return hueShift(c[ind], shift * .2) * (1. + shift * .3);
}


void main() {
    if (v_quad_pos.y > 1) discard;
    commonInit();
    float u_padding = u_padding * UNITS_SCALE;
    const float EDGE_RADIUS = 0.5;
    const float SOLID_BORDER = 0.07;
    vec2 uv = vec2(cos(v_quad_pos.x * pi * 2),sin(v_quad_pos.x * pi * 2)) * min(v_quad_pos.y, 1 - EDGE_RADIUS);

    vec4 col = vec4(0);
    float h = clanColorHash();
    uv = rotateCW(uv,h * pi * 2);
    h = (h * 2 - 1) * (h * 2 - 1);

    vec2 p = h * 2. + (0.1 + h) * uv;
    
    // domains
    
    float r = sqrt(dot(p,p));
    float a = cos(p.y * p.x);
           
    // distortion
    
    a += fbm(vec2(1.9 - p.x, .3 * u_time + p.y + u_random * 5));
    a += fbm(3.4 * p + u_random);
    r += fbm(-0.9 * p);
    
    // colorize
    
    float ff = 1.0 - smoothstep(-0.4, 1.2, noise(vec2(0.5 * a, 9.3 * a)) );
    col = vec4(stepClanColor(fract(ff + r)),1);
    if (v_quad_pos.y > 1 - SOLID_BORDER) {
        col.rgb = parent_faction_color;
    }
    else if (v_quad_pos.y > 1 - EDGE_RADIUS - SOLID_BORDER) {
        col.a = smoothstep(1, 1 - EDGE_RADIUS, v_quad_pos.y);
    }

    gl_FragColor = col;
}
#endif