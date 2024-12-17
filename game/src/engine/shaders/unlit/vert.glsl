#version 300 es

layout(location = 0) in vec3 vert_position;
layout(location = 1) in vec2 vert_texcoord;

out vec2 frag_texcoord;

uniform mat4 model;
uniform mat4 view;
uniform mat4 proj;

void main() {
    gl_Position = proj * view * model * vec4(vert_position, 1.0);
    frag_texcoord = vert_texcoord;
}
