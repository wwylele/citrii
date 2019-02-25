#version 330 core

uniform vec4 coord;
uniform vec4 tex_coord;

out vec2 f_tex_coord;

void main() {
    vec2 c;
    if (gl_VertexID == 0) {
        f_tex_coord = tex_coord.xy;
        c = coord.xy;
    } else if (gl_VertexID == 1) {
        f_tex_coord = tex_coord.zy;
        c = coord.zy;
    } else if (gl_VertexID == 2) {
        f_tex_coord = tex_coord.xw;
        c = coord.xw;
    } else {
        f_tex_coord = tex_coord.zw;
        c = coord.zw;
    }
    gl_Position = vec4(c, 0.0, 1.0);
}
