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
uniform sampler2D u_frame_texture;
in vec2 v_quad_pos;

float factionColorMatch(vec3 color) {
    const float THRESHOLD = 0.01;
    return float(
        min(distance(color, player_faction_color), distance(color, enemy_faction_color)) < THRESHOLD
        );
}

void main() {
    vec2 textureSize = textureSize(u_frame_texture, 0);
    vec4 col = texture(u_frame_texture, gl_FragCoord.xy / textureSize);
    col.rgb *= factionColorMatch(col.rgb);
    col.a = 1;
    gl_FragColor = col;
}
#endif