#version 330 core

in vec4 coord_world;
in vec4 normal_world;
in vec2 texcoord;

layout(location = 0) out vec4 out_color;

uniform vec3 camera_pos;
uniform vec3 light_source;

uniform vec4 base_color;
uniform mat4 color_tran[5];
uniform int tex_mode[5]; // 0: normal; 1: windowed; 2: windowed with mirror
uniform mat3 tex_tran[5];
uniform sampler2D tex0;
uniform sampler2D tex1;
uniform sampler2D tex2;
uniform sampler2D tex3;
uniform sampler2D tex4;


vec4 layer(vec4 base, int i, sampler2D top) {
    vec2 coord = texcoord;
    if (tex_mode[i] == 2 && coord.x > 0.5) {
        coord.x = 1.0 - coord.x;
    }

    if (tex_mode[i] != 0) {
        coord = (tex_tran[i] * vec3(coord, 1.0)).xy;
        if (any(lessThan(coord, vec2(0.0))) || any(greaterThan(coord, vec2(1.0)))) {
            return base;
        }
    }

    vec4 tex_origin = texture(top, coord);
    vec4 tex_color = color_tran[i] * vec4(tex_origin.xyz, 1.0);
    tex_color.a = min(tex_color.a, tex_origin.a);
    return vec4(base.xyz * (1.0 - tex_color.w) + tex_color.xyz * tex_color.w, max(base.w, tex_color.w));
}

void main() {
    vec3 v = normalize(camera_pos - coord_world.xyz);
    vec3 l = normalize(light_source - coord_world.xyz);
    vec3 n = normalize(normal_world.xyz);
    vec3 h = normalize(l + v);

    vec4 mixed_color = base_color;
    mixed_color = layer(mixed_color, 0, tex0);
    mixed_color = layer(mixed_color, 1, tex1);
    mixed_color = layer(mixed_color, 2, tex2);
    mixed_color = layer(mixed_color, 3, tex3);
    mixed_color = layer(mixed_color, 4, tex4);

    if (mixed_color.w < 0.01) discard;

    float diffuse = max(0.0, dot(n, l)) * 0.2 + 0.8;
    float specular = 0.3 * pow(max(0.0, dot(n, h)), 5);

    out_color = vec4(clamp(mixed_color.xyz * diffuse + specular * vec3(1.0), 0.0, 1.0), mixed_color.w);
}
