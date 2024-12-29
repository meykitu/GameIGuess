#version 330 core

layout(location = 0) in vec3 in_pos;
layout(location = 1) in vec2 in_tex_coord;

out vec2 texcoord;

uniform mat4 mvp;
uniform float time; // Time uniform for animation

void main() {
    texcoord = in_tex_coord;

    gl_Position = mvp * vec4(in_pos, 1.0);
}
