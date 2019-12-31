#version 140

uniform mat4 mat;

in vec2 pos;
in vec2 texture_uv;

out vec2 f_texture_uv;

void main() {
    gl_Position = mat * vec4(pos, 0.0, 1.0);
    f_texture_uv = texture_uv;
}