#version 150

uniform mat4 mat;

in vec3 pos;
in vec2 texture_uv;
in vec3 normal;
in vec4 color;

out vec3 f_normal;
out vec2 f_texture_uv;
out vec4 f_color;

void main() {
    gl_Position = mat * vec4(pos, 1.0);
    f_normal = transpose(inverse(mat3(mat))) * normal;
    f_texture_uv = texture_uv;
    f_color = color;
}
