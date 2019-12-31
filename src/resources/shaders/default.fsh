#version 140

in vec3 f_normal;
in vec4 f_color;

out vec4 out_color;

void main() {
    if (f_color.a == 0.0) discard;
    out_color = f_color;
}