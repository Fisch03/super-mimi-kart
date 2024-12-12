#version 300 es
precision mediump float;

in vec2 frag_texcoord;

out vec4 color;

uniform sampler2D tex;

uniform uvec2 sprite_size;
uniform uint sprite_sheet_size;
uniform uint sprite_index;

void main() {
    vec2 sprite_size_normalized = vec2(sprite_size) / vec2(sprite_sheet_size);
    vec2 sprite_offset = vec2(sprite_size_normalized.x * float(sprite_index), 0.0);

    color = texture(tex, frag_texcoord + sprite_offset);

    // color = vec4(frag_texcoord + sprite_offset, 0.0, 0.5);
}
