#version 120

varying vec3 f_normal;
varying vec4 f_color;

void main() {
    if (f_color.a == 0.0) discard;
    gl_FragColor = f_color;
}