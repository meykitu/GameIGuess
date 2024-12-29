#version 330 core

in vec2 texcoord;

out vec4 fragColor;

uniform sampler2D tex;

void main() {
    vec4 tex_color = texture(tex, texcoord);
    
    // Create a variation factor based on texcoord
    float variation = 0.5 + 0.1 * sin(texcoord.x * 2.0 + texcoord.y * 2.0);
    
    // Apply the variation to darken the texture
    vec4 varied_color = tex_color;
    
    fragColor = varied_color;
}
