#version 300 es 
precision mediump float;

in vec3 frag_texcoord;

out vec4 color;

uniform samplerCube cube;

void main()
{    
    vec3 coord = frag_texcoord - vec3(0.0, -0.05, 0.0);
    color = texture(cube, coord);
}
