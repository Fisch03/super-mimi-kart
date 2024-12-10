#version 300 es
precision mediump float;

in vec3 position;
out vec4 color;

void main() {
    color = vec4(position, 1.0f);
}