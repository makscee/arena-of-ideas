#include <common.glsl>

#ifdef VERTEX_SHADER
out vec2 v_quad_pos;
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;
void main() {
    float radius = u_unit_radius * c_units_scale;
    float height = 0.1 + radius * .5 + (radius * .5 * a_pos.y);
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

vec3 getMixedAllianceColor(float t) {
    vec3 col = colors[0];
    if (u_clan_count == 2) {
        col = mix2Colors(t, vec3[2](colors[0],colors[1]));
    } else if (u_clan_count == 3) {
        col = mix3Colors(t, colors);
    }
    return col;
}

const vec3 ORANGE = vec3(1.0, 0.6, 0.2);
const vec3 PINK   = vec3(0.7, 0.1, 0.4); 
const vec3 BLUE   = vec3(0.0, 0.2, 0.9); 
const vec3 BLACK  = vec3(0.0, 0.0, 0.2);

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

float generateOffsetFromColor(vec3 color, float addOffset) {
    return fract(color.r + color.g + color.b + addOffset);
}

void main() {
    commonInit();
    float u_padding = u_padding * c_units_scale;
    vec2 uv = v_quad_pos;
    vec3 mixedColor = getMixedAllianceColor(uv.x);
    if (uv.y > 1) {
        gl_FragColor = vec4(mixedColor,0.6 * smoothstep(1.0 + u_padding * 2., 1, uv.y));
        return;
    }

    const float maxOffset = 1.5;
    float offsetX = generateOffsetFromColor(colors[0], 0) * maxOffset;
    float offsetY = generateOffsetFromColor(colors[0], 0);
    if (u_clan_count > 1) {
        offsetY = generateOffsetFromColor(colors[1], offsetY);
    }
    if (u_clan_count > 2) {
        offsetY = generateOffsetFromColor(colors[2], offsetY) * maxOffset;
    }
    uv = vec2(cos(uv.x * pi * 2),sin(uv.x * pi * 2)) * uv.y + vec2(offsetX, offsetY);
    vec2 p = -1. + 1. * uv;
    
    // domains
    
    float r = sqrt(dot(p,p));
    float a = cos(p.y * p.x);
           
    // distortion
    
    float f = fbm(5.0 * p);
    a += fbm(vec2(1.9 - p.x, 0.5 * u_time + p.y));
    a += fbm(0.4 * p);
    r += fbm(2.9 * p);
    
    // colorize
    
    vec3 col = colors[0];
    
    float ff = 1.0 - smoothstep(-0.4, 1.1, noise(vec2(0.5 * a, 3.3 * a)) );
    col =  mix( col, colors[1], ff);
       
    ff = 1.0 - smoothstep(.0, 2.8, r );
    col +=  mix( col, BLACK,  ff);
    
    ff -= 1.0 - smoothstep(0.3, 0.5, fbm(vec2(1.0, 40.0 * a)) ); 
    col =  mix( col, colors[2],  ff);  
      
    ff = 1.0 - smoothstep(2., 2.9, a * 1.5 ); 
    col =  mix( col, BLACK,  ff);
    if (v_quad_pos.y > 0.9) {
        gl_FragColor = alphaBlend(vec4(col,0.5),vec4(mixedColor,0.5));
        return;
    }
    gl_FragColor = vec4(col,1);
}
#endif