#version 120

attribute vec3 f_normal;
attribute vec4 f_color;

varying vec4 out_color;

void main() {
    if (f_color.a == 0.0) discard;
    out_color = f_color;
}