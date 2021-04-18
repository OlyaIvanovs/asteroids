#version 330 core

in vec2 tex_coords;
out vec4 Color;

uniform sampler2D ourTexture;

void main() {
    vec3 color = texture(ourTexture, tex_coords).rgb;
    Color = vec4(color * sin(tex_coords.x), 1.0);
}