#version 140

uniform sampler2D tex;

in vec2 f_texture_uv;
in vec3 f_normal;
in vec4 f_color;

out vec4 out_color;

void main() {
    if (f_color.a == 0.0) discard;
    out_color = texture2D(tex, f_texture_uv) * f_color;
}