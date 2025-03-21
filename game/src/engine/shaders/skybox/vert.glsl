#version 300 es 

layout (location = 0) in vec3 vert_position;

out vec3 frag_texcoord;

uniform mat4 view;
uniform mat4 proj;

void main()
{
    frag_texcoord = vert_position;
    vec4 pos = proj * view * vec4(vert_position, 1.0);
    gl_Position = pos.xyww;
}  
