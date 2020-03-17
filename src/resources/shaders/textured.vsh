#version 120

uniform mat4 mat;

attribute vec3 pos;
attribute vec2 texture_uv;
attribute vec3 normal;
attribute vec4 color;

varying vec3 f_normal;
varying vec2 f_texture_uv;
varying vec4 f_color;

void main() {
    gl_Position = mat * vec4(pos, 1.0);
    f_normal = normal;
    f_texture_uv = texture_uv;
    f_color = color;
}
