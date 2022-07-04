#define CIRCLE 1
#define HEART 2

uniform vec2 p_velocity = vec2(0,1);
uniform float p_startRadius = 0.15;
uniform float p_endRadius = -0.1;
uniform int p_count = 15;
uniform vec4 p_startColor = vec4(0.862, 0.078, 0.235, 1);
uniform vec4 p_endColor = vec4(0.117, 0.564, 1, 1);
uniform float p_lifeTime = 1;
uniform float p_startAlpha = 1;
uniform float p_endAlpha = 1;
uniform int p_shape = 1;
uniform vec2 p_gravity = vec2(0, 0);