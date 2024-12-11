#version 300 es
precision mediump float;

in vec2 frag_texcoord;

out vec4 color;

uniform sampler2D tex;

void main() {
    color = texture(tex, frag_texcoord);
    // color = vec4(1.0, 0.0, 0.0, 1.0);
}
