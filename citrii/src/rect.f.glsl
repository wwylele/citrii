#version 330 core

uniform vec4 color;
uniform sampler2D tex;

in vec2 f_tex_coord;

layout(location = 0) out vec4 out_color;

void main() {
    if (color.w == 0.0) out_color = texture(tex, f_tex_coord) * vec4(color.xyz, 1.0);
    else out_color = color;
}
