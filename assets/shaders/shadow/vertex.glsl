#version 330 core

layout(location = 0) in vec3 in_pos;
layout(location = 1) in vec2 in_tex_coord;

uniform mat4 mvp;
uniform mat4 light_mvp;

out vec2 tex_coord;
out vec4 frag_pos_light_space;

void main() {
    tex_coord = in_tex_coord;
    frag_pos_light_space = light_mvp * vec4(in_pos, 1.0);
    gl_Position = mvp * vec4(in_pos, 1.0);
}