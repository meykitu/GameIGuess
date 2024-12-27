#version 330 core

in vec2 tex_coord;
in vec4 frag_pos_light_space;

uniform sampler2D texture;
uniform sampler2D shadow_map;

out vec4 frag_color;

float shadow_calculation(vec4 frag_pos_light_space) {
    vec3 proj_coords = frag_pos_light_space.xyz / frag_pos_light_space.w;
    proj_coords = proj_coords * 0.5 + 0.5;
    float closest_depth = texture2D(shadow_map, proj_coords.xy).r;
    float current_depth = proj_coords.z;
    float shadow = current_depth > closest_depth + 0.005 ? 1.0 : 0.0;
    return shadow;
}

void main() {
    vec4 tex_color = texture2D(texture, tex_coord);
    float shadow = shadow_calculation(frag_pos_light_space);
    vec3 lighting = vec3(1.0) * (1.0 - shadow);
    frag_color = vec4(tex_color.rgb * lighting, tex_color.a);
}