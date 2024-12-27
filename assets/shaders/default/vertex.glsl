#version 130

in vec3 in_pos;
in vec2 in_tex_coord;

out vec2 texcoord;
out vec4 frag_pos_light_space;

uniform mat4 mvp;
uniform mat4 light_mvp;

void main() {
    texcoord = in_tex_coord;
    frag_pos_light_space = light_mvp * vec4(in_pos, 1.0);
    gl_Position = mvp * vec4(in_pos, 1.0);
}