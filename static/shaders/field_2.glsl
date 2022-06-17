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

#define SCALE 2.0
#define TAU (PI*2.0)
#define CL(x,a,b) smoothstep(0.0,1.0,(2.0/3.0)*(x-a)/(b-a)+(1.0/6.0))*(b-a)+a // https://www.shadertoy.com/view/Ws3Xzr

void main() {
    vec2 fragCoord = (v_quad_pos * u_window_size + u_window_size / 2);

    float st = radians(-31.0); // start time
    // float t = st+(u_time*TAU)/3600.0;
    float t = 1.3;
    float n = (cos(t) > 0.0) ? sin(t): 1.0/sin(t);
    float z = pow(500.0, n); // autozoom
    z = clamp(z, CL(z, 1e-16, 1e-15), CL(z, 1e+17, 1e+18)); // clamp to prevent black screen
    vec2 uv = (fragCoord-0.5*u_window_size.xy)/u_window_size.y*SCALE*z;
    float ls = (u_time*TAU)/5.0; // light animation speed
    float a = atan(uv.x, -uv.y); // screen arc
    float i = a/TAU; // spiral increment 0.5 per 180°
    float r = pow(length(uv), 0.5/n)-i; // archimedean at 0.5
    float cr = ceil(r); // round up radius
    float wr = cr+i; // winding ratio
    float vd = (cr*TAU+a) / (n*2.0); // visual denominator
    float vd2 = vd*2.0;
    vec3 col = vec3(sin(wr*vd+ls)); // blend it
    col *= pow(sin(fract(r)*PI), floor(abs(n*2.0))+5.0); // smooth edges
    col *= sin(vd2*wr+PI/2.0+ls*2.0); // this looks nice
    col *= 0.2+abs(cos(vd2)); // dark spirals
    vec3 g = mix(vec3(0), vec3(1), pow(length(uv)/z, -1.0/n)); // dark gradient
    col = min(col, g); // blend gradient with spiral
    vec3 rgb = vec3( cos(vd2)+1.0, abs(sin(t)), cos(PI+vd2)+1.0 );
    col += (col*2.0)-(rgb*0.5); // add color
    col *= .15;
    gl_FragColor = vec4(col, 1.0);
}
#endif