#version 300 es
precision mediump float;

in vec2 frag_texcoord;

out vec4 color;

uniform sampler2D tex;

uniform uint sprite_amount;
uniform uint sprite_index;

void main() {
    vec2 sprite_uv = frag_texcoord / vec2(sprite_amount, 1.0) + vec2(float(sprite_index) / float(sprite_amount), 0.0);
    
    color = texture(tex, sprite_uv);
}
