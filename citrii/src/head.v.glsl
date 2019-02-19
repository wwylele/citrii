#version 330 core
layout(location = 0) in vec3 in_coord;
layout(location = 1) in vec3 in_normal;
layout(location = 2) in vec2 in_texcoord;

out vec4 coord_world;
out vec4 normal_world;
out vec2 texcoord;

uniform mat4 object_tran;
uniform mat4 object_tran_inv;
uniform mat4 camera_tran;

void main() {
    vec3 coord = in_coord / 256.0;
    vec3 normal = in_normal / 256.0;
    texcoord = in_texcoord / 8192.0;

    vec4 world_pos = object_tran * vec4(coord, 1.0);
    normal_world = transpose(object_tran_inv) * vec4(normal, 0.0);
    coord_world = world_pos;
    gl_Position = camera_tran * world_pos;
}
