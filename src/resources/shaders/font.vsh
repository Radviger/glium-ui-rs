#version 140

uniform mat4 mat;

attribute vec2 pos;
attribute vec2 texture_uv;

varying vec2 f_texture_uv;

void main() {
    gl_Position = mat * vec4(pos, 0.0, 1.0);
    f_texture_uv = texture_uv;
}