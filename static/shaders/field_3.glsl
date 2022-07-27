#include <common.glsl>

uniform ivec2 u_window_size;
#ifdef VERTEX_SHADER
attribute vec2 a_pos;
uniform mat3 u_projection_matrix;
uniform mat3 u_view_matrix;

out vec2 v_quad_pos;
flat out int p_index;
flat out float faction;
void main() {
    v_quad_pos = a_pos;
    p_index = gl_InstanceID;
    faction = mix(-1,1,float(p_index % 2));

    float ratio = float(u_window_size.y) / float(u_window_size.x);
    float t = fract(u_time * .1 + rand(p_index + 3));
    vec2 aabb = a_pos * vec2(u_unit_radius * ratio, u_unit_radius) * (t - 1);

    vec2 startPos = vec2(faction, (rand(p_index) - 0.5) * 2) + aabb;
    vec2 velocity = vec2(-faction,0) * .9 * rand(p_index + 10) - vec2(.2) * faction * cos(t * 2.5) * 2;
    vec2 pos = startPos + velocity * t;

    gl_Position = vec4(pos, 0.0, 1);
}
#endif

#ifdef FRAGMENT_SHADER
uniform sampler2D u_previous_texture;
in vec2 v_quad_pos;
flat in float faction;

void main() {
    // vec2 fragCoord = (v_quad_pos * u_window_size + u_window_size / 2);
    vec2 uv = v_quad_pos;
    float dist = length(uv);
    if (dist > 1) discard;
    vec3 col = mix(player_faction_color, enemy_faction_color, (faction + 1) / 2);

    gl_FragColor = vec4(col, 1);
}
#endif