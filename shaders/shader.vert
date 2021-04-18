#version 330 core

layout(location = 0) in vec3 Position;
layout(location = 1) in vec2 Texture_Coords;

out vec2 tex_coords;

uniform mat4 Projection;
uniform mat4 View;
uniform mat4 Model;

void main() {
    gl_Position = Projection * View * Model * vec4(Position, 1.0);
    tex_coords = Texture_Coords;
}